use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use http::{request::Request, response::Response};
use logger::fields::FieldValue;
use tokio::sync::watch;
use tonic::body::BoxBody;
use tonic::server::NamedService;
use tonic::transport::Server;

use crate::{definition, env, plugin};
use crate::service::context::Context;
use crate::errors as merrors;

pub trait GrpcService: Send + 'static {}

#[derive(Clone)]
pub(crate) struct Grpc<S> {
    port: i32,
    server: S,
}

impl<S> Grpc<S>
where
    S: tonic::codegen::Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
        + NamedService
        + Clone
        + Send
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
impl<S> plugin::service::Service for Grpc<S>
where
    S: tonic::codegen::Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
        + NamedService
        + Clone
        + Send
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
        ]
    }

    async fn run(&mut self, _ctx: &Context, shutdown_rx: watch::Receiver<()>) -> merrors::Result<()> {
        let addr2 = format!("127.0.0.1:{}", self.port).parse().unwrap();
        let shutdown_signal = async {
            let mut shutdown_rx = shutdown_rx.clone();

            // Wait until the receiver sees the shutdown signal
            shutdown_rx.changed().await.ok();
        };

        let _ = Server::builder()
            .add_service(self.server.clone())
            .serve_with_shutdown(addr2, shutdown_signal)
            .await;

        Ok(())
    }

    async fn stop(&mut self, _ctx: &Context) {
        println!("stop service");
        println!("port: {}", self.port);
    }
}