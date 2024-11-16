use std::any::Any;
use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use mikros::FutureMutex;
use mikros::http::{ServiceInternalState, ServiceState};
use mikros::service::builder::ServiceBuilder;
//use mikros::FutureMutex;

#[derive(Clone, Default)]
pub struct AppState {
    value: i32,
}

impl AppState {
    pub fn increase(&mut self) {
        self.value += 1;
    }
}

impl ServiceInternalState for AppState {
    fn clone_box(&self) -> Box<dyn ServiceInternalState> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Handler method for the first endpoint
//async fn handler_one(ctx: Option<State<Arc<Context>>>) -> String {
async fn handler_one(State(state): State<Arc<FutureMutex<ServiceState>>>) -> String {
    println!("Handler One");
    let context = state.lock().await.context();
    context.logger().info("just a log message");

    format!("Handler One")
}

// Handler method for the second endpoint
async fn handler_two(State(state): State<Arc<FutureMutex<ServiceState>>>) -> String {
    println!("Handler Two");

    if let Some(app_state) = state.lock().await.state::<AppState>().await {
        println!("App State current value: {}", app_state.value);
        app_state.increase();
        let mut x = app_state.clone_box();
        x.in
    }

    format!("Handler Two")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = axum::Router::new()
        .route("/one", get(handler_one))
        .route("/two", get(handler_two));

    let state = AppState::default();
    let svc = ServiceBuilder::default()
        .http_with_state(api, Box::new(state))
//        .http_with_state(api, Arc::new(FutureMutex::new(state)))
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
