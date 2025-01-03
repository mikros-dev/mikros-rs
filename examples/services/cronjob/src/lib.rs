pub mod builder;

use std::collections::HashMap;
use std::sync::Arc;

use mikros::definition::ServiceKind;
use mikros::env::Env;
use mikros::{errors as merrors, plugin, Mutex, serde_json};
use mikros::service::context::Context;
use mikros::service::lifecycle::Lifecycle;
use serde_derive::Deserialize;

#[async_trait::async_trait]
pub trait CronjobService: Send + Sync {
    async fn handler(&mut self, ctx: &Context) -> merrors::Result<()>;
}

#[derive(Clone)]
pub struct Cronjob {
    service: Arc<Mutex<Box<dyn CronjobService>>>,
    definitions: Option<Definitions>,
}

#[derive(Clone, Deserialize)]
struct Definitions {
    frequency: String,
    days: Vec<String>,
    scheduled_times: Vec<String>,
}

impl Cronjob {
    pub fn new(svc: Box<dyn CronjobService>) -> Self {
        Self {
            service: Arc::new(Mutex::new(svc)),
            definitions: None,
        }
    }
}

impl Lifecycle for Cronjob {}

#[async_trait::async_trait]
impl plugin::service::Service for Cronjob {
    fn kind(&self) -> ServiceKind {
        ServiceKind::Custom("cronjob".into())
    }

    fn info(&self) -> serde_json::Value {
        match self.definitions {
            Some(ref d) => serde_json::json!({
                "frequency": d.frequency,
                "days": d.days,
                "scheduled_times": d.scheduled_times,
            }),
            None => serde_json::json!({}),
        }
    }

    fn initialize(&mut self, definitions: Arc<mikros::definition::Definitions>, _envs: Arc<Env>, _: HashMap<String, serde_json::Value>) -> merrors::Result<()> {
        self.definitions = definitions.load_service(self.kind())?;
        if self.definitions.is_none() {
            // TODO: return error here
        }

        Ok(())
    }

    async fn run(&self, ctx: &Context, _shutdown_rx: tokio::sync::watch::Receiver<()>) -> merrors::Result<()> {
        // A real cronjob service would schedule the task to execute using
        // definitions settings. We just call the handler...
        self.service.lock().await.handler(ctx).await
    }

    async fn stop(&self, _ctx: &Context) {
        // noop
    }

    fn mode(&self) -> plugin::service::ServiceExecutionMode {
        plugin::service::ServiceExecutionMode::NonBlock
    }
}
