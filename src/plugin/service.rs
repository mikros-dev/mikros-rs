use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::watch;

use crate::service::context::Context;
use crate::service::lifecycle::Lifecycle;
use crate::{definition, env, errors};

/// Service is a set of methods that every new kind of service must implement
/// to be supported by the framework.
#[async_trait::async_trait]
pub trait Service: ServiceClone + Lifecycle {
    /// The new service kind. Usually the ServiceKind::Custom should be used
    /// for new services implementation.
    fn kind(&self) -> definition::ServiceKind;

    /// Returns internal information about the service implementation to be
    /// logged when the service is being initialized.
    fn info(&self) -> serde_json::Value;

    /// Defines which type of execution the service implementation uses.
    fn mode(&self) -> ServiceExecutionMode;

    /// Initializes everything that the service implementation needs to run.
    ///
    /// Here is the only place where values can be stored inside the implementation
    /// for later usage.
    fn initialize(
        &mut self,
        ctx: Arc<Context>,
        definitions: Arc<definition::Definitions>,
        envs: Arc<env::Env>,
        options: HashMap<String, serde_json::Value>,
    ) -> Result<(), errors::ServiceError>;

    /// Puts the service implementation to run.
    async fn run(
        &self,
        ctx: Arc<Context>,
        shutdown_rx: watch::Receiver<()>,
    ) -> Result<(), errors::ServiceError>;

    /// Stops the current service implementation. The place to let the service
    /// execute its graceful shutdown.
    async fn stop(&self, ctx: Arc<Context>);
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
