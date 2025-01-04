use std::sync::Arc;

use axum::extract::State;

use crate::http::ServiceState;
use crate::Mutex;

// The default /health handler for every application.
pub(crate) async fn handler(State(_): State<Arc<Mutex<ServiceState>>>) -> String {
    "".to_string()
}
