use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use mikros::service::builder::ServiceBuilder;
use mikros::service::context::Context;

#[derive(Clone)]
pub struct Service {
}

impl Service {
    pub fn new() -> Self {
        Self {}
    }
}

// Handler method for the first endpoint
//async fn handler_one(ctx: Option<State<Arc<Context>>>) -> String {
async fn handler_one() -> String {
    println!("Handler One");
    format!("Handler One")
}

// Handler method for the second endpoint
async fn handler_two(State(ctx): State<Arc<Context>>) -> String {
    println!("Handler Two");
    format!("Handler Two")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let x = axum::Router::new()
        .route("/one", get(handler_one));
//        .route("/two", get(handler_two));
    
    let service = Service::new();
    let svc = ServiceBuilder::default()
        .http(x)
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
