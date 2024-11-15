use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use mikros::http::{ServiceInternalState, ServiceState};
use mikros::service::builder::ServiceBuilder;
use mikros::FutureMutex;

#[derive(Clone, Default)]
pub struct AppState;
impl ServiceInternalState for AppState {}

// Handler method for the first endpoint
//async fn handler_one(ctx: Option<State<Arc<Context>>>) -> String {
async fn handler_one(State(state): State<Arc<ServiceState>>) -> String {
    println!("Handler One");
    let context = state.context();
    context.logger().info("just a log message");

    format!("Handler One")
}

// Handler method for the second endpoint
async fn handler_two(State(_state): State<Arc<ServiceState>>) -> String {
    println!("Handler Two");
    format!("Handler Two")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = axum::Router::new()
        .route("/one", get(handler_one))
        .route("/two", get(handler_two));

    let state = AppState::default();
    let svc = ServiceBuilder::default()
        .http_with_state(api, Arc::new(FutureMutex::new(state)))
//        .http(api)
        .build();

    match svc {
        Ok(mut svc) => {
            if let Err(e) = svc.start().await {
                println!("application error: {}", e);
            }
        },
        Err(e) => panic!("{}", e.to_string())
    }

    Ok(())
}
