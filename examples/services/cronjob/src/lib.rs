pub mod builder;

use std::collections::HashMap;
use std::sync::Arc;

use mikros::definition::ServiceKind;
use mikros::env::Env;
use mikros::service::context::Context;
use mikros::service::lifecycle::Lifecycle;
use mikros::{async_trait, errors, plugin, serde_json, Mutex};

#[async_trait::async_trait]
pub trait CronjobService: Send + Sync {
    async fn handler(&mut self, ctx: Arc<Context>) -> errors::Result<()>;
}

#[derive(Clone)]
pub struct Cronjob {
    service: Arc<Mutex<Box<dyn CronjobService>>>,
    definitions: Option<Definitions>,
}

#[derive(Clone, mikros::Deserialize)]
#[serde(crate = "mikros::serde")]
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

    fn mode(&self) -> plugin::service::ServiceExecutionMode {
        plugin::service::ServiceExecutionMode::NonBlock
    }

    fn initialize(
        &mut self,
        _: Arc<Context>,
        definitions: Arc<mikros::definition::Definitions>,
        _: Arc<Env>,
        _: HashMap<String, serde_json::Value>,
    ) -> errors::Result<()> {
        self.definitions = definitions.load_service(self.kind());
        if self.definitions.is_none() {
            // TODO: return error here?
        }

        Ok(())
    }

    async fn run(
        &self,
        ctx: Arc<Context>,
        _shutdown_rx: mikros::tokio::sync::watch::Receiver<()>,
    ) -> errors::Result<()> {
        // A real cronjob service would schedule the task to execute using
        // definitions settings. We just call the handler...
        self.service.lock().await.handler(ctx.clone()).await
    }

    async fn stop(&self, _ctx: Arc<Context>) {
        // noop
    }
}
