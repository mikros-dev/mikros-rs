use std::collections::HashMap;
use std::sync::Arc;

use axum::Router;
use futures::lock::Mutex;
use logger::fields::FieldValue;
use tokio::net::TcpListener;
use tokio::sync::watch::Receiver;

use crate::{definition, env, errors as merrors, plugin};
use crate::http::{ServiceInternalState, ServiceState};
use crate::service::context::Context;
use crate::service::lifecycle::Lifecycle;

#[derive(Clone)]
pub(crate) struct Http {
    port: i32,
    router: Router<Arc<ServiceState>>,
    lifecycle: Option<Box<Arc<Mutex<dyn Lifecycle>>>>,
    internal_state: Option<Box<Arc<Mutex<dyn ServiceInternalState>>>>
}

impl Http {
    pub fn new(router: Router<Arc<ServiceState>>) -> Self {
        Self {
            port: 0,
            router,
            lifecycle: None,
            internal_state: None
        }
    }

    pub fn new_with_lifecycle<L: Lifecycle + 'static>(router: Router<Arc<ServiceState>>, lifecycle: Arc<Mutex<L>>) -> Self {
        Self {
            port: 0,
            router,
            lifecycle: Some(Box::new(lifecycle)),
            internal_state: None
        }
    }

    pub fn new_with_state<S: ServiceInternalState + 'static>(router: Router<Arc<ServiceState>>, state: Arc<Mutex<S>>) -> Self {
        Self {
            port: 0,
            router,
            lifecycle: None,
            internal_state: Some(Box::new(state))
        }
    }

    pub fn new_with_lifecycle_and_state<L: Lifecycle + 'static, S: ServiceInternalState + 'static>(router: Router<Arc<ServiceState>>, lifecycle: Arc<Mutex<L>>, state: Arc<Mutex<S>>) -> Self {
        Self {
            port: 0,
            router,
            lifecycle: Some(Box::new(lifecycle)),
            internal_state: Some(Box::new(state))
        }
    }
}

#[async_trait::async_trait]
impl Lifecycle for Http {
    async fn on_start(&mut self) -> merrors::Result<()> {
        if let Some(lifecycle) = &self.lifecycle {
            return lifecycle.lock().await.on_start().await
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

    fn initialize(&mut self, envs: Arc<env::Env>, definitions: Arc<definition::Definitions>) -> merrors::Result<()> {
        let service_type = definitions.get_service_type(definition::ServiceKind::Http)?;
        self.port = match service_type.1 {
            None => envs.http_port,
            Some(port) => port,
        };

        Ok(())
    }

    fn info(&self) -> HashMap<String, FieldValue> {
        logger::fields![
            "svc.port" => FieldValue::Number(self.port as i64),
            "svc.mode" => FieldValue::String(definition::ServiceKind::Http.to_string()),
        ]
    }

    async fn run(&self, ctx: &Context, shutdown_rx: Receiver<()>) -> merrors::Result<()> {
        let addr = format!("0.0.0.0:{}", self.port);
        let shutdown_signal = async move {
            let mut shutdown_rx = shutdown_rx.clone();

            // Wait until the receiver sees the shutdown signal
            shutdown_rx.changed().await.ok();
        };

        let state = match &self.internal_state {
            None => ServiceState::new(ctx),
            Some(st) => ServiceState::new_with_state(ctx, st)
        };

        let app = Router::new().merge(self.router.clone()).with_state(Arc::new(state));

        match TcpListener::bind(addr).await {
            Ok(incoming) => {
                if let Err(e) = axum::serve(incoming, app).with_graceful_shutdown(shutdown_signal).await {
                    return Err(merrors::Error::InternalServiceError(format!("could not initialize http server: {}", e)))
                }

                Ok(())
            }
            Err(e) => Err(merrors::Error::InternalServiceError(format!("could not initialize http server: {}", e)))
        }
    }

    async fn stop(&self, _ctx: &Context) {
        // noop
    }
}