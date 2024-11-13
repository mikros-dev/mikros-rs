use std::collections::HashMap;
use std::sync::Arc;
use logger::fields::FieldValue;

use crate::{definition, env, plugin};
use crate::service::context::Context;
use crate::errors as merrors;

pub trait GrpcService: Send + 'static {}

#[derive(Clone)]
pub(crate) struct Grpc {
    port: i32,
}

impl Grpc {
    pub(crate) fn new() -> Self {
        Self {
            port: 0,
        }
    }
}

#[async_trait::async_trait]
impl plugin::service::Service for Grpc {
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

    async fn run(&mut self, _ctx: &Context) -> merrors::Result<()> {
        todo!()
    }

    async fn stop(&mut self, _ctx: &Context) {
        todo!()
    }
}