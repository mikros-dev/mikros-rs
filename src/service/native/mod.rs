
use std::sync::{Arc, Mutex};

use crate::{errors as merrors, plugin};
use crate::service::context::Context;

pub trait NativeService: NativeServiceClone {
    fn start(&mut self, ctx: &Context) -> merrors::Result<()>;
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

impl plugin::service::Service for Native {
    fn name(&self) -> &str {
        "native"
    }

    fn initialize(&mut self) -> merrors::Result<()> {
        Ok(())
    }

    fn run(&mut self, ctx: &Context) -> merrors::Result<()> {
        self.svc.lock().unwrap().start(ctx)
    }

    fn stop(&mut self, ctx: &Context) {
        self.svc.lock().unwrap().stop(ctx)
    }
}