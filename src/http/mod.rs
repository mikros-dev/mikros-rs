use std::any::Any;
use std::sync::Arc;

use futures::lock::Mutex;

use crate::service::context::Context;

#[derive(Clone)]
pub struct ServiceState {
    context: Arc<Context>,

    /// This member gives access to the service own state (added when it is
    /// created, with the ServiceBuilder::http_with_state API).
    ///
    /// One can retrieve the proper service state structure like the example:
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// use axum::extract::State;
    /// use mikros::Mutex;
    /// use mikros::http::ServiceState;
    ///
    /// #[derive(Clone)]
    /// pub struct AppState;
    ///
    /// async fn handler(State(state): State<Arc<Mutex<ServiceState>>>) -> String {
    ///     if let Some(app_state) = &state.lock().await.app_state {
    ///         let mut locked = app_state.as_ref().lock().await;
    ///         let svc_state = locked.downcast_mut::<AppState>().unwrap();
    ///
    ///         // svc_state can be manipulated from here.
    ///     }
    ///
    ///     "Ok".to_string()
    /// }
    /// ```
    ///
    pub app_state: Option<Arc<Mutex<dyn Any + Send + Sync>>>,
}

impl ServiceState {
    pub(crate) fn new(context: &Context) -> Self {
        Self {
            context: Arc::new(context.clone()),
            app_state: None,
        }
    }

    pub(crate) fn new_with_state(context: &Context, internal_state: Arc<Mutex<dyn Any + Send + Sync>>) -> Self {
        let mut s = Self::new(context);
        s.app_state = Some(internal_state.clone());
        s
    }

    /// Allows retrieving the mikros Context object from a handler state. Even
    /// if the service was not initialized with state, the context will always
    /// be available.
    pub fn context(&self) -> Arc<Context> {
        self.context.clone()
    }
}
