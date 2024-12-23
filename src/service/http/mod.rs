mod errors;
mod health;

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use futures::lock::Mutex;
use tokio::net::TcpListener;
use tokio::sync::watch::Receiver;

use crate::http::ServiceState;
use crate::service::context::Context;
use crate::service::lifecycle::Lifecycle;
use crate::{definition, env, errors as merrors, plugin};
use crate::plugin::service::ServiceExecutionMode;

#[derive(Clone)]
pub(crate) struct Http {
    port: i32,
    internal_health_handler: bool,
    router: Router<Arc<Mutex<ServiceState>>>,
    lifecycle: Option<Arc<Mutex<dyn Lifecycle>>>,
    app_state: Option<Arc<Mutex<dyn Any + Send + Sync>>>,
}

impl Http {
    pub fn new(router: Router<Arc<Mutex<ServiceState>>>) -> Self {
        Self {
            port: 0,
            internal_health_handler: true,
            router,
            lifecycle: None,
            app_state: None,
        }
    }

    pub fn new_with_lifecycle<L>(router: Router<Arc<Mutex<ServiceState>>>, lifecycle: Arc<Mutex<L>>) -> Self
    where
        L: Lifecycle + 'static,
    {
        let mut s = Self::new(router);
        s.lifecycle = Some(lifecycle.clone());
        s
    }

    pub fn new_with_state(router: Router<Arc<Mutex<ServiceState>>>, state: Arc<Mutex<dyn Any + Send + Sync>>) -> Self {
        let mut s = Self::new(router);
        s.app_state = Some(state.clone());
        s
    }

    pub fn new_with_lifecycle_and_state<L>(router: Router<Arc<Mutex<ServiceState>>>, lifecycle: Arc<Mutex<L>>, state: Arc<Mutex<dyn Any + Send + Sync>>) -> Self
    where
        L: Lifecycle + 'static,
    {
        let mut s = Self::new(router);
        s.lifecycle = Some(lifecycle.clone());
        s.app_state = Some(state.clone());
        s
    }

    // Builds the application router according user builder options.
    fn router(&self, ctx: &Context) -> Router {
        let state = match &self.app_state {
            None => ServiceState::new(ctx),
            Some(st) => ServiceState::new_with_state(ctx, st.clone())
        };

        // Create the server router.
        let mut router = Router::new();

        if self.internal_health_handler {
            router = router.route("/health", get(health::handler))
        }

        router
            .merge(self.router.clone())
            .with_state(Arc::new(Mutex::new(state)))
    }
}

#[async_trait::async_trait]
impl Lifecycle for Http {
    async fn on_start(&mut self, ctx: &Context) -> merrors::Result<()> {
        if let Some(lifecycle) = &self.lifecycle {
            return lifecycle.lock().await.on_start(ctx).await
        }

        Ok(())
    }

    async fn on_finish(&self) -> merrors::Result<()> {
        if let Some(lifecycle) = &self.lifecycle {
            return lifecycle.lock().await.on_finish().await
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl plugin::service::Service for Http {
    fn kind(&self) -> definition::ServiceKind {
        definition::ServiceKind::Http
    }

    fn initialize(&mut self, definitions: Arc<definition::Definitions>, envs: Arc<env::Env>, options: HashMap<String, serde_json::Value>) -> merrors::Result<()> {
        // Store the service port to listen for.
        let service_type = definitions.get_service_type(definition::ServiceKind::Http)?;
        self.port = match service_type.1 {
            None => envs.http_port,
            Some(port) => port,
        };

        // Store if we're going to use the default health handler or not.
        if let Some(health_endpoint) = options.get("without_health_endpoint") {
            self.internal_health_handler = !health_endpoint.as_bool().unwrap_or(false);
        }

        Ok(())
    }

    fn info(&self) -> serde_json::Value {
        serde_json::json!({
            "svc.port": self.port,
            "svc.mode": definition::ServiceKind::Http.to_string(),
        })
    }

    fn mode(&self) -> ServiceExecutionMode {
        ServiceExecutionMode::Block
    }

    async fn run(&self, ctx: &Context, shutdown_rx: Receiver<()>) -> Result<(), merrors::Error> {
        let addr = format!("0.0.0.0:{}", self.port);
        let shutdown_signal = async move {
            let mut shutdown_rx = shutdown_rx.clone();

            // Wait until the receiver sees the shutdown signal
            shutdown_rx.changed().await.ok();
        };

        match TcpListener::bind(addr).await {
            Err(e) => Err(errors::Error::InitFailure(e.to_string()).into()),
            Ok(incoming) => {
                if let Err(e) = axum::serve(incoming, self.router(ctx)).with_graceful_shutdown(shutdown_signal).await {
                    return Err(errors::Error::ShutdownFailure(e.to_string()).into())
                }

                Ok(())
            }
        }
    }

    async fn stop(&self, _ctx: &Context) {
        // noop
    }
}