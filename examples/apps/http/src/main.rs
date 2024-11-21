use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use futures::lock::Mutex;
use mikros::http::ServiceState;
use mikros::service::builder::ServiceBuilder;

// Handler method for the first endpoint
async fn handler_one(State(state): State<Arc<Mutex<ServiceState>>>) -> String {
    println!("Handler One");
    let context = state.lock().await.context();
    context.logger().info("just a log message");

    format!("Handler One")
}

// Handler method for the second endpoint
async fn handler_two(State(_): State<Arc<Mutex<ServiceState>>>) -> String {
    println!("Handler Two");
    format!("Handler Two")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = axum::Router::new()
        .route("/one", get(handler_one))
        .route("/two", get(handler_two));

    let svc = ServiceBuilder::default().http(api).build();

    match svc {
        Ok(mut svc) => svc.start().await,
        Err(e) => panic!("{}", e.to_string()),
    }

    Ok(())
}
