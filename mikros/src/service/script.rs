use std::collections::HashMap;
use std::sync::Arc;

use futures::lock::Mutex;
use tokio::sync::watch;

use crate::plugin::service::ServiceExecutionMode;
use crate::service::context::Context;
use crate::service::lifecycle::Lifecycle;
use crate::{definition, env, errors, plugin};

#[async_trait::async_trait]
pub trait ScriptService: ScriptServiceClone + Lifecycle + Send {
    async fn run(&self, ctx: Arc<Context>) -> errors::Result<()>;
    async fn cleanup(&self, ctx: Arc<Context>);
}

pub trait ScriptServiceClone {
    fn clone_dyn(&self) -> Box<dyn ScriptService>;
}

impl<T> ScriptServiceClone for T
where
    T: 'static + ScriptService + Clone,
{
    fn clone_dyn(&self) -> Box<dyn ScriptService> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn ScriptService> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

#[derive(Clone)]
pub(crate) struct Script {
    svc: Arc<Mutex<Box<dyn ScriptService>>>,
}

impl Script {
    pub fn new(svc: Box<dyn ScriptService>) -> Self {
        Self {
            svc: Arc::new(Mutex::new(svc)),
        }
    }
}

#[async_trait::async_trait]
impl Lifecycle for Script {
    async fn on_start(&mut self, ctx: Arc<Context>) -> errors::Result<()> {
        self.svc.lock().await.on_start(ctx).await
    }

    async fn on_finish(&self) -> errors::Result<()> {
        self.svc.lock().await.on_finish().await
    }
}

#[async_trait::async_trait]
impl plugin::service::Service for Script {
    fn kind(&self) -> definition::ServiceKind {
        definition::ServiceKind::Script
    }

    fn info(&self) -> serde_json::Value {
        serde_json::json!({
            "kind": self.kind().to_string(),
        })
    }

    fn mode(&self) -> ServiceExecutionMode {
        ServiceExecutionMode::NonBlock
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

    async fn run(&self, ctx: Arc<Context>, _: watch::Receiver<()>) -> errors::Result<()> {
        self.svc.lock().await.run(ctx).await
    }

    async fn stop(&self, ctx: Arc<Context>) {
        self.svc.lock().await.cleanup(ctx).await;
    }
}
