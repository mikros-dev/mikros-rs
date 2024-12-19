use std::sync::Arc;

use tokio::sync::watch;

use crate::{definition, env, errors};
use crate::service::context::Context;
use crate::service::lifecycle::Lifecycle;

#[async_trait::async_trait]
pub trait Service: ServiceClone + Lifecycle {
    fn kind(&self) -> definition::ServiceKind;
    fn initialize(&mut self, envs: Arc<env::Env>, definitions: Arc<definition::Definitions>) -> Result<(), errors::Error>;
    fn info(&self) -> serde_json::Value;
    fn mode(&self) -> ServiceExecutionMode;

    async fn run(&self, ctx: &Context, shutdown_rx: watch::Receiver<()>) -> Result<(), errors::Error>;
    async fn stop(&self, ctx: &Context);
}

pub trait ServiceClone {
    fn clone_box(&self) -> Box<dyn Service>;
}

impl<T> ServiceClone for T
where
    T: 'static + Service + Clone,
{
    fn clone_box(&self) -> Box<dyn Service> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Service> {
    fn clone(&self) -> Self {
        ServiceClone::clone_box(self.as_ref())
    }
}

#[derive(PartialEq)]
pub enum ServiceExecutionMode {
    Block,
    NonBlock,
}
