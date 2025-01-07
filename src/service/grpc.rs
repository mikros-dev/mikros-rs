mod errors;

use std::collections::HashMap;
use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use futures::lock::Mutex;
use http::{request::Request, response::Response};
use tokio::sync::watch;
use tonic::body::BoxBody;
use tonic::server::NamedService;
use tonic::transport::Server;

use crate::errors as merrors;
use crate::grpc;
use crate::service::context::Context;
use crate::service::lifecycle::Lifecycle;
use crate::{definition, env, plugin};

#[derive(Clone)]
pub(crate) struct Grpc<S> {
    port: i32,
    server: S,
    lifecycle: Option<Arc<Mutex<dyn Lifecycle>>>,
}

impl<S> Grpc<S>
where
    S: tonic::codegen::Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
        + NamedService
        + Clone
        + Send
        + Sync
        + 'static,
    S::Future: Send + 'static,
{
    pub(crate) fn new_with_lifecycle<L: Lifecycle + 'static>(
        server: S,
        lifecycle: Arc<Mutex<L>>,
    ) -> Self {
        Self {
            port: 0,
            server,
            lifecycle: Some(lifecycle.clone()),
        }
    }

    pub(crate) fn new(server: S) -> Self {
        Self {
            port: 0,
            server,
            lifecycle: None,
        }
    }
}

#[async_trait::async_trait]
impl<S> Lifecycle for Grpc<S>
where
    S: tonic::codegen::Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
        + NamedService
        + Clone
        + Send
        + Sync
        + 'static,
    S::Future: 'static + Send,
{
    async fn on_start(&mut self, ctx: Arc<Context>) -> merrors::Result<()> {
        if let Some(lifecycle) = &self.lifecycle {
            return lifecycle.lock().await.on_start(ctx).await;
        }

        Ok(())
    }

    async fn on_finish(&self) -> merrors::Result<()> {
        if let Some(lifecycle) = &self.lifecycle {
            return lifecycle.lock().await.on_finish().await;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl<S> plugin::service::Service for Grpc<S>
where
    S: tonic::codegen::Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
        + NamedService
        + Clone
        + Send
        + Sync
        + 'static,
    S::Future: Send + 'static,
{
    fn kind(&self) -> definition::ServiceKind {
        definition::ServiceKind::Grpc
    }

    fn initialize(
        &mut self,
        ctx: Arc<Context>,
        definitions: Arc<definition::Definitions>,
        envs: Arc<env::Env>,
        _: HashMap<String, serde_json::Value>,
    ) -> merrors::Result<()> {
        match definitions.get_service_type(definition::ServiceKind::Grpc) {
            Err(e) => return Err(merrors::ServiceError::from_error(ctx.clone(), e.into())),
            Ok(service_type) => {
                self.port = match service_type.1 {
                    None => envs.grpc_port,
                    Some(port) => port,
                }
            }
        }

        Ok(())
    }

    fn info(&self) -> serde_json::Value {
        serde_json::json!({
            "svc.port": self.port,
            "svc.mode": definition::ServiceKind::Grpc.to_string(),
        })
    }

    fn mode(&self) -> plugin::service::ServiceExecutionMode {
        plugin::service::ServiceExecutionMode::Block
    }

    async fn run(
        &self,
        ctx: Arc<Context>,
        shutdown_rx: watch::Receiver<()>,
    ) -> merrors::Result<()> {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), self.port as u16);

        let shutdown_signal = async {
            let mut shutdown_rx = shutdown_rx.clone();

            // Wait until the receiver sees the shutdown signal
            shutdown_rx.changed().await.ok();
        };

        let layer = tower::ServiceBuilder::new()
            .layer(grpc::ContextExtractor::new(ctx.clone()))
            .into_inner();

        if let Err(e) = Server::builder()
            .layer(layer)
            .add_service(self.server.clone())
            .serve_with_shutdown(addr, shutdown_signal)
            .await
        {
            let grpc_error = errors::Error::TransportInitFailure(e.to_string());

            return Err(merrors::ServiceError::internal(
                ctx.clone(),
                &grpc_error.description(),
            ));
        }

        Ok(())
    }

    async fn stop(&self, _: Arc<Context>) {
        // noop
    }
}
