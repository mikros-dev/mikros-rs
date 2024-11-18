use std::collections::HashMap;
use std::sync::{Arc};

use futures::lock::Mutex;
use logger::fields::FieldValue;
use tokio::sync::watch;

use crate::{definition, env, errors as merrors, plugin};
use crate::service::context::Context;
use crate::service::lifecycle::Lifecycle;

#[async_trait::async_trait]
pub trait ScriptService: ScriptServiceClone + Lifecycle + Send {
    fn run(&self, ctx: &Context) -> merrors::Result<()>;
    fn cleanup(&self, ctx: &Context);
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
    async fn on_start(&mut self) -> merrors::Result<()> {
        self.svc.lock().await.on_start().await
    }

    async fn on_finish(&self) -> merrors::Result<()> {
        self.svc.lock().await.on_finish().await
    }
}

#[async_trait::async_trait]
impl plugin::service::Service for Script {
    fn kind(&self) -> definition::ServiceKind {
        definition::ServiceKind::Script
    }

    fn initialize(&mut self, _: Arc<env::Env>, _: Arc<definition::Definitions>) -> merrors::Result<()> {
        Ok(())
    }

    fn info(&self) -> HashMap<String, FieldValue> {
        logger::fields![
            "kind".to_string() => FieldValue::String(self.kind().to_string()),
        ]
    }

    async fn run(&self, ctx: &Context, _: watch::Receiver<()>) -> merrors::Result<()> {
        self.svc.lock().await.run(ctx)
    }

    async fn stop(&self, ctx: &Context) {
        self.svc.lock().await.cleanup(ctx)
    }
}
