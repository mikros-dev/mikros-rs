use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use logger::fields::FieldValue;
use tokio::sync::watch;

use crate::{definition, env, errors as merrors, plugin};
use crate::service::context::Context;

pub trait ScriptService: Send + ScriptServiceClone {
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
    svc: Box<Arc<Mutex<dyn ScriptService>>>,
}

impl Script {
    pub fn new<S: ScriptService + 'static>(svc: Arc<Mutex<S>>) -> Self {
        Self {
            svc: Box::new(svc),
        }
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
        self.svc.lock().unwrap().run(ctx)
    }

    async fn stop(&self, ctx: &Context) {
        self.svc.lock().unwrap().cleanup(ctx)
    }
}