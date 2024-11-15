use std::sync::Arc;

use futures::lock::Mutex;

use crate::service::context::Context;

pub trait ServiceInternalState: Send + 'static {}

#[derive(Clone)]
pub struct ServiceState {
    context: Arc<Context>,
    state: Option<Box<Arc<Mutex<dyn ServiceInternalState>>>>
}

impl ServiceState {
    pub(crate) fn new(context: &Context) -> ServiceState {
        Self {
            context: Arc::new(context.clone()),
            state: None
        }
    }

    pub(crate) fn new_with_state(context: &Context, state: &Box<Arc<Mutex<dyn ServiceInternalState>>>) -> ServiceState {
        Self {
            context: Arc::new(context.clone()),
            state: Some(state.clone())
        }
    }

    pub fn context(&self) -> Arc<Context> {
        self.context.clone()
    }

    pub fn state(&self) -> Option<Box<Arc<Mutex<dyn ServiceInternalState>>>> {
        self.state.clone()
    }
}