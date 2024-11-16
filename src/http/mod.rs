use std::any::Any;
use std::sync::Arc;

use futures::lock::Mutex;

use crate::service::context::Context;

#[derive(Clone)]
pub struct ServiceState {
    context: Arc<Context>,
    state: Arc<Mutex<Option<Box<dyn ServiceInternalState>>>>
}

impl ServiceState {
    pub(crate) fn new(context: &Context) -> Self {
        Self {
            context: Arc::new(context.clone()),
            state: Arc::new(Mutex::new(None)),
        }
    }

    pub(crate) fn new_with_state(context: &Context, state: Box<dyn ServiceInternalState>) -> Self {
        Self {
            context: Arc::new(context.clone()),
            state: Arc::new(Mutex::new(Some(state))),
        }
    }

    pub fn context(&self) -> Arc<Context> {
        self.context.clone()
    }

    pub async fn state<T: Clone + 'static>(&self) -> Option<Arc<T>> {
        let state = self.state.lock().await;
        state.as_ref()?.as_any().downcast_ref::<T>().map(|t|Arc::new(t.clone()))
    }
}

pub trait ServiceInternalState: Send + Sync + 'static {
    fn clone_box(&self) -> Box<dyn ServiceInternalState>;
    fn as_any(&self) -> &dyn Any;
}

impl Clone for Box<dyn ServiceInternalState> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
