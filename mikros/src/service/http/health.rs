use std::sync::Arc;

use axum::extract::State;

use crate::Mutex;
use crate::http::ServiceState;

// The default /health handler for every application.
pub(crate) async fn handler(State(_): State<Arc<Mutex<ServiceState>>>) -> String {
    String::new()
}
