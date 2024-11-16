use std::collections::HashMap;
use std::ops::Deref;
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
    router: Router<Arc<Mutex<ServiceState>>>,
    lifecycle: Option<Box<Arc<Mutex<dyn Lifecycle>>>>,
    internal_state: Arc<Mutex<Option<Box<dyn ServiceInternalState>>>>
}

impl Http {
    pub fn new(router: Router<Arc<Mutex<ServiceState>>>) -> Self {
        Self {
            port: 0,
            router,
            lifecycle: None,
            internal_state: Arc::new(Mutex::new(None))
        }
    }

    pub fn new_with_lifecycle<L>(router: Router<Arc<Mutex<ServiceState>>>, lifecycle: Arc<Mutex<L>>) -> Self
    where
        L: Lifecycle + 'static,
    {
        let mut s = Self::new(router);
        s.lifecycle = Some(Box::new(lifecycle));
        s
    }

    pub fn new_with_state(router: Router<Arc<Mutex<ServiceState>>>, state: Box<dyn ServiceInternalState>) -> Self {
        let mut s = Self::new(router);
        s.internal_state = Arc::new(Mutex::new(Some(state)));
        s
    }

    pub fn new_with_lifecycle_and_state<L>(router: Router<Arc<Mutex<ServiceState>>>, lifecycle: Arc<Mutex<L>>, state: Box<dyn ServiceInternalState>) -> Self
    where
        L: Lifecycle + 'static,
    {
        let mut s = Self::new(router);
        s.lifecycle = Some(Box::new(lifecycle));
        s.internal_state = Arc::new(Mutex::new(Some(state)));
        s
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

        let internal_state = self.internal_state.clone();
        let state = match internal_state.lock().await.deref() {
            None => ServiceState::new(ctx),
            Some(state) => ServiceState::new_with_state(ctx, state.clone_box())
        };

        let app = Router::new().merge(self.router.clone()).with_state(Arc::new(Mutex::new(state)));

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