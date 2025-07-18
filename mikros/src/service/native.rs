use std::collections::HashMap;
use std::sync::Arc;

use futures::lock::Mutex;
use tokio::sync::watch;

use crate::plugin::service::ServiceExecutionMode;
use crate::service::context::Context;
use crate::service::lifecycle::Lifecycle;
use crate::{definition, env, errors, plugin};

#[async_trait::async_trait]
pub trait NativeService: NativeServiceClone + Lifecycle + Send + Sync {
    /// This is the place where the service/application must be initialized. It
    /// should do the required initialization, put jobs to execute in the
    /// background and finish. It shouldn't block.
    async fn start(&mut self, ctx: Arc<Context>) -> errors::Result<()>;

    /// The stop callback is called when the service/application is requested
    /// to finish. It must be responsible for finishing any previously started
    /// job.
    async fn stop(&self, ctx: Arc<Context>);
}

pub trait NativeServiceClone {
    fn clone_dyn(&self) -> Box<dyn NativeService>;
}

impl<T> NativeServiceClone for T
where
    T: 'static + NativeService + Clone,
{
    fn clone_dyn(&self) -> Box<dyn NativeService> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn NativeService> {
    fn clone(&self) -> Box<dyn NativeService> {
        self.clone_dyn()
    }
}

#[derive(Clone)]
pub(crate) struct Native {
    svc: Arc<Mutex<Box<dyn NativeService>>>,
}

impl Native {
    pub fn new(svc: Box<dyn NativeService>) -> Self {
        Self {
            svc: Arc::new(Mutex::new(svc)),
        }
    }
}

#[async_trait::async_trait]
impl Lifecycle for Native {
    async fn on_start(&mut self, ctx: Arc<Context>) -> errors::Result<()> {
        self.svc.lock().await.on_start(ctx).await
    }

    async fn on_finish(&self) -> errors::Result<()> {
        self.svc.lock().await.on_finish().await
    }
}

#[async_trait::async_trait]
impl plugin::service::Service for Native {
    fn kind(&self) -> definition::ServiceKind {
        definition::ServiceKind::Native
    }

    fn info(&self) -> serde_json::Value {
        serde_json::json!({
            "kind": self.kind().to_string(),
        })
    }

    fn mode(&self) -> ServiceExecutionMode {
        ServiceExecutionMode::Block
    }

    fn initialize(
        &mut self,
        _: Arc<Context>,
        _: Arc<definition::Definitions>,
        _: Arc<env::Env>,
        _: HashMap<String, serde_json::Value>,
    ) -> errors::Result<()> {
        Ok(())
    }

    async fn run(&mut self, ctx: Arc<Context>, _: watch::Receiver<()>) -> errors::Result<()> {
        self.svc.lock().await.start(ctx).await
    }

    async fn stop(&self, ctx: Arc<Context>) {
        self.svc.lock().await.stop(ctx).await;
    }
}
