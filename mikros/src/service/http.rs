mod errors;
mod health;

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use futures::lock::Mutex;
use tokio::net::TcpListener;
use tokio::sync::watch::Receiver;

use crate::http::ServiceState;
use crate::plugin::service::ServiceExecutionMode;
use crate::service::context::Context;
use crate::service::lifecycle::Lifecycle;
use crate::{definition, env, errors as merrors, plugin};

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

    pub fn new_with_lifecycle<L>(
        router: Router<Arc<Mutex<ServiceState>>>,
        lifecycle: Arc<Mutex<L>>,
    ) -> Self
    where
        L: Lifecycle + 'static,
    {
        let mut s = Self::new(router);
        s.lifecycle = Some(lifecycle);
        s
    }

    pub fn new_with_state(
        router: Router<Arc<Mutex<ServiceState>>>,
        state: Arc<Mutex<dyn Any + Send + Sync>>,
    ) -> Self {
        let mut s = Self::new(router);
        s.app_state = Some(state);
        s
    }

    pub fn new_with_lifecycle_and_state<L>(
        router: Router<Arc<Mutex<ServiceState>>>,
        lifecycle: Arc<Mutex<L>>,
        state: Arc<Mutex<dyn Any + Send + Sync>>,
    ) -> Self
    where
        L: Lifecycle + 'static,
    {
        let mut s = Self::new(router);
        s.lifecycle = Some(lifecycle);
        s.app_state = Some(state);
        s
    }

    // Builds the application router according user builder options.
    fn router(&self, ctx: Arc<Context>) -> Router {
        let state = match &self.app_state {
            None => ServiceState::new(ctx),
            Some(st) => ServiceState::new_with_state(ctx, st.clone()),
        };

        // Create the server router.
        let mut router = Router::new();

        if self.internal_health_handler {
            router = router.route("/health", get(health::handler));
        }

        router
            .merge(self.router.clone())
            .with_state(Arc::new(Mutex::new(state)))
    }
}

#[async_trait::async_trait]
impl Lifecycle for Http {
    async fn on_start(&mut self, ctx: Arc<Context>) -> merrors::Result<()> {
        if let Some(lifecycle) = &self.lifecycle {
            return lifecycle.lock().await.on_start(ctx).await;
        }

        Ok(())
    }

    async fn on_finish(&self) -> merrors::Result<()> {
        if let Some(lifecycle) = &self.lifecycle {
            return lifecycle.lock().await.on_finish().await;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl plugin::service::Service for Http {
    fn kind(&self) -> definition::ServiceKind {
        definition::ServiceKind::Http
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

    fn initialize(
        &mut self,
        ctx: Arc<Context>,
        definitions: Arc<definition::Definitions>,
        envs: Arc<env::Env>,
        options: HashMap<String, serde_json::Value>,
    ) -> merrors::Result<()> {
        // Store the service port to listen for.
        match definitions.get_service_type(definition::ServiceKind::Http) {
            Err(e) => return Err(merrors::ServiceError::from_error(ctx.clone(), e.into())),
            Ok(service_type) => {
                self.port = match service_type.1 {
                    None => envs.http_port,
                    Some(port) => port,
                }
            }
        }

        // Store if we're going to use the default health handler or not.
        if let Some(health_endpoint) = options.get("without_health_endpoint") {
            self.internal_health_handler = !health_endpoint.as_bool().unwrap_or(false);
        }

        Ok(())
    }

    async fn run(&self, ctx: Arc<Context>, shutdown_rx: Receiver<()>) -> merrors::Result<()> {
        let addr = format!("0.0.0.0:{}", self.port);
        let shutdown_signal = async move {
            let mut shutdown_rx = shutdown_rx.clone();

            // Wait until the receiver sees the shutdown signal
            shutdown_rx.changed().await.ok();
        };

        match TcpListener::bind(addr).await {
            Err(e) => {
                let http_error = errors::Error::InitFailure(e.to_string());
                Err(merrors::ServiceError::from_error(
                    ctx.clone(),
                    http_error.into(),
                ))
            }
            Ok(incoming) => {
                if let Err(e) = axum::serve(incoming, self.router(ctx.clone()))
                    .with_graceful_shutdown(shutdown_signal)
                    .await
                {
                    let http_error = errors::Error::ShutdownFailure(e.to_string());
                    return Err(merrors::ServiceError::from_error(
                        ctx.clone(),
                        http_error.into(),
                    ));
                }

                Ok(())
            }
        }
    }

    async fn stop(&self, _: Arc<Context>) {
        // noop
    }
}
