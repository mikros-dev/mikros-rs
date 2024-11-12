use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use logger::fields::FieldValue;

use crate::{definition, errors as merrors, plugin};
use crate::service::context::Context;

#[async_trait::async_trait]
pub trait NativeService: Send + NativeServiceClone {
    /// This is the place where the service/application must be initialized. It
    /// should do the required initialization, put any job to execute in background
    /// and leave. It shouldn't block.
    fn start(&mut self, ctx: &Context) -> merrors::Result<()>;

    /// The stop callback is called when the service/application is requested
    /// to finish. It must be responsible for finishing any previously started
    /// job.
    fn stop(&mut self, ctx: &Context);
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
    svc: Box<Arc<Mutex<dyn NativeService>>>,
}

impl Native {
    pub fn new<S: NativeService + 'static>(svc: Arc<Mutex<S>>) -> Self {
        Self {
            svc: Box::new(svc),
        }
    }
}

#[async_trait::async_trait]
impl plugin::service::Service for Native {
    fn kind(&self) -> definition::ServiceKind {
        definition::ServiceKind::Native
    }

    fn initialize(&mut self) -> merrors::Result<()> {
        Ok(())
    }

    async fn run(&mut self, ctx: &Context) -> merrors::Result<()> {
        self.svc.lock().unwrap().start(ctx)
    }

    async fn stop(&mut self, ctx: &Context) {
        self.svc.lock().unwrap().stop(ctx)
    }

    fn information(&self) -> HashMap<String, FieldValue> {
        logger::fields![
            "kind".to_string() => FieldValue::String(self.kind().to_string()),
        ]
    }
}