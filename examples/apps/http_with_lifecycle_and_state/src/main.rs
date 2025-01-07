use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use futures::lock::Mutex;
use mikros::errors::ServiceError;
use mikros::http::ServiceState;
use mikros::service::builder::ServiceBuilder;
use mikros::service::lifecycle::Lifecycle;

#[derive(Clone, Default)]
pub struct AppState {
    value: i32,
}

impl AppState {
    pub fn increase(&mut self) {
        self.value += 1;
    }
}

#[async_trait::async_trait]
impl Lifecycle for AppState {
    async fn on_start(
        &mut self,
        _ctx: Arc<mikros::service::context::Context>,
    ) -> Result<(), ServiceError> {
        println!("service on_start");
        self.value = 42;
        Ok(())
    }

    async fn on_finish(&self) -> Result<(), ServiceError> {
        println!("service on_finish");
        Ok(())
    }
}

// Handler method for the first endpoint
async fn handler_one(State(state): State<Arc<Mutex<ServiceState>>>) -> String {
    println!("Handler One");
    let context = state.lock().await.context();
    context.logger().info("just a log message");

    format!("Handler One")
}

// Handler method for the second endpoint
async fn handler_two(State(state): State<Arc<Mutex<ServiceState>>>) -> String {
    println!("Handler Two");

    let context = state.lock().await.context();
    if let Some(app_state) = &state.lock().await.app_state {
        let mut locked = app_state.as_ref().lock().await;
        let x = locked.downcast_mut::<AppState>().unwrap();
        x.value += 1;
        context
            .logger()
            .info(format!("value: {}", x.value).as_str());
    }

    format!("Handler Two")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = axum::Router::new()
        .route("/one", get(handler_one))
        .route("/two", get(handler_two));

    let state = Arc::new(Mutex::new(AppState::default()));
    let mut svc = ServiceBuilder::default()
        .http_with_lifecycle_and_state(api, state.clone(), state.clone())
        .build()?;

    Ok(svc.start().await?)
}
