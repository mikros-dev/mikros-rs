use std::collections::HashMap;
use std::sync::Arc;

use futures::lock::Mutex;
use logger::fields::FieldValue;
use tokio::sync::watch;

use crate::service::context::Context;
use crate::service::lifecycle::Lifecycle;
use crate::{definition, env, errors as merrors, plugin};
use crate::plugin::service::ServiceExecutionMode;

#[async_trait::async_trait]
pub trait NativeService: NativeServiceClone + Lifecycle + Send + Sync {
    /// This is the place where the service/application must be initialized. It
    /// should do the required initialization, put any job to execute in background
    /// and leave. It shouldn't block.
    async fn start(&self, ctx: &Context) -> merrors::Result<()>;

    /// The stop callback is called when the service/application is requested
    /// to finish. It must be responsible for finishing any previously started
    /// job.
    async fn stop(&self, ctx: &Context);
}

pub trait NativeServiceClone {
    fn clone_dyn(&self) -> Box<dyn NativeService>;
}

impl<T> NativeServiceClone for T
where
    T: 'static + NativeService + Clone
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
    async fn on_start(&mut self) -> merrors::Result<()> {
        self.svc.lock().await.on_start().await
    }

    async fn on_finish(&self) -> merrors::Result<()> {
        self.svc.lock().await.on_finish().await
    }
}

#[async_trait::async_trait]
impl plugin::service::Service for Native {
    fn kind(&self) -> definition::ServiceKind {
        definition::ServiceKind::Native
    }

    fn initialize(&mut self, _: Arc<env::Env>, _: Arc<definition::Definitions>) -> merrors::Result<()> {
        Ok(())
    }

    fn info(&self) -> HashMap<String, FieldValue> {
        logger::fields![
            "kind".to_string() => FieldValue::String(self.kind().to_string()),
        ]
    }

    fn mode(&self) -> ServiceExecutionMode {
        ServiceExecutionMode::Block
    }

    async fn run(&self, ctx: &Context, _: watch::Receiver<()>) -> merrors::Result<()> {
        self.svc.lock().await.start(ctx).await
    }

    async fn stop(&self, ctx: &Context) {
        self.svc.lock().await.stop(ctx).await
    }
}