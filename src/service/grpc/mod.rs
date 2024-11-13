use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use http::{request::Request, response::Response};
use logger::fields::FieldValue;
use tokio::sync::watch;
use tonic::body::BoxBody;
use tonic::server::NamedService;
use tonic::transport::Server;

use crate::{definition, env, plugin};
use crate::service::context::Context;
use crate::errors as merrors;
use crate::grpc;
use crate::service::lifecycle::Lifecycle;

#[derive(Clone)]
pub(crate) struct Grpc<S> {
    port: i32,
    server: S,
}

impl<S> Grpc<S>
where
    S: tonic::codegen::Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
        + NamedService
        + Lifecycle
        + Clone
        + Send
        + Sync
        + 'static,
    S::Future: Send + 'static,
{
    pub(crate) fn new(svc: S) -> Self {
        Self {
            port: 0,
            server: svc,
        }
    }
}

#[async_trait::async_trait]
impl<S> Lifecycle for Grpc<S>
where
    S: tonic::codegen::Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
        + NamedService
        + Lifecycle
        + Clone
        + Send
        + Sync
        + 'static,
    S::Future: 'static + Send,
{
    async fn on_start(&mut self) -> merrors::Result<()> {
        self.server.on_start().await
    }

    async fn on_finish(&self) -> merrors::Result<()> {
        self.server.on_finish().await
    }
}

#[async_trait::async_trait]
impl<S> plugin::service::Service for Grpc<S>
where
    S: tonic::codegen::Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
        + NamedService
        + Lifecycle
        + Clone
        + Send
        + Sync
        + 'static,
    S::Future: Send + 'static,
{
    fn kind(&self) -> definition::ServiceKind {
        definition::ServiceKind::Grpc
    }

    fn initialize(&mut self, envs: Arc<env::Env>, definitions: Arc<definition::Definitions>) -> merrors::Result<()> {
        let service_type = definitions.get_service_type(definition::ServiceKind::Grpc)?;
        self.port = match service_type.1 {
            None => envs.grpc_port,
            Some(port) => port,
        };

        Ok(())
    }

    fn info(&self) -> HashMap<String, FieldValue> {
        logger::fields![
            "svc.port" => FieldValue::Number(self.port as i64),
            "svc.mode" => FieldValue::String(definition::ServiceKind::Grpc.to_string()),
        ]
    }

    async fn run(&self, ctx: &Context, shutdown_rx: watch::Receiver<()>) -> merrors::Result<()> {
        let addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            self.port as u16,
        );

        let shutdown_signal = async {
            let mut shutdown_rx = shutdown_rx.clone();

            // Wait until the receiver sees the shutdown signal
            shutdown_rx.changed().await.ok();
        };

        let layer = tower::ServiceBuilder::new()
            .layer(grpc::ContextExtractor::new(ctx))
            .into_inner();

        if let Err(e) = Server::builder()
            .layer(layer)
            .add_service(self.server.clone())
            .serve_with_shutdown(addr, shutdown_signal)
            .await
        {
            return Err(merrors::Error::InternalServiceError(format!("could not initialize grpc server: {}", e)))
        }

        Ok(())
    }

    async fn stop(&self, _ctx: &Context) {
        // noop
    }
}