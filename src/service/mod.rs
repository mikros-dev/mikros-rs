pub mod builder;
pub mod context;
pub mod grpc;
pub mod http;
pub mod lifecycle;
pub mod native;
pub mod script;
mod errors;

use std::collections::HashMap;
use std::sync::Arc;

use tokio::signal;
use tokio::sync::watch;
use tokio::task::{self, JoinHandle};

use crate::args::Args;
use crate::definition::{Definitions, ServiceKind, CustomServiceInfo};
use crate::{errors as merrors, logger, plugin};
use crate::env::Env;
use crate::plugin::service::ServiceExecutionMode;
use crate::service::builder::ServiceBuilder;

pub struct Service {
    envs: Arc<Env>,
    definitions: Arc<Definitions>,
    logger: Arc<logger::Logger>,
    servers: HashMap<String, Box<dyn plugin::service::Service>>,
    context: context::Context,
    handlers: Vec<JoinHandle<()>>,
    shutdown_tx: watch::Sender<()>,
}

impl Service {
    pub(crate) fn new(builder: ServiceBuilder) -> merrors::Result<Self> {
        let definitions = Service::load_definitions(&builder)?;
        let logger = Self::start_logger(&definitions);
        let (shutdown_tx, _) = watch::channel(());
        let envs = Env::load(&definitions)?;

        let mut features = vec![];
        for f in builder.features.clone() {
            features.push(f);
        }

        Ok(Service {
            envs: envs.clone(),
            definitions: definitions.clone(),
            logger: logger.clone(),
            context: Self::build_context(envs.clone(), logger, definitions, features)?,
            servers: builder.servers,
            handlers: Vec::new(),
            shutdown_tx,
        })
    }

    fn load_definitions(builder: &ServiceBuilder) -> merrors::Result<Arc<Definitions>> {
        let args = Args::load();
        let custom_info = if !builder.custom_service_types.is_empty() {
            Some(CustomServiceInfo{
                types: Some(builder.custom_service_types.clone()),
            })
        } else {
            None
        };

        Ok(Definitions::new(args.config_path.as_deref(), custom_info)?)
    }

    fn start_logger(defs: &Definitions) -> Arc<logger::Logger> {
        let log = defs.log();

        Arc::new(logger::builder::LoggerBuilder::new()
            .with_level(log.level.parse::<logger::Level>().unwrap_or(logger::Level::Info))
            .with_local_timestamp(log.local_timestamp)
            .with_field("svc.name", &defs.name)
            .with_field("svc.version", &defs.version)
            .with_field("svc.product", &defs.product)
            .with_field("svc.language", &defs.language)
            .build())
    }

    fn build_context(
        envs: Arc<Env>,
        logger: Arc<logger::Logger>,
        defs: Arc<Definitions>,
        features: Vec<Box<dyn plugin::feature::Feature>>,
    ) -> merrors::Result<context::Context> {
        context::Context::new(envs, logger, defs, features)
    }

    /// Puts the service to run.
    pub async fn start(&mut self) -> merrors::Result<()> {
        self.logger.info("service starting");
        self.start_service().await
    }

    async fn start_service(&mut self) -> merrors::Result<()> {
        self.validate_definitions()?;
        self.start_features().await?;
        self.initialize_service_internals().await?;
        self.print_service_resources().await;
        self.run().await
    }

    fn validate_definitions(&self) -> merrors::Result<()> {
        if self.servers.is_empty() {
            return Err(errors::Error::EmptyServiceFound.into());
        }

        // Script services should be a single service.
        if !self.has_equal_execution_modes() {
            return Err(errors::Error::UnsupportedServicesExecutionMode.into());
        }

        for t in &self.definitions.types {
            if !self.servers.contains_key(&t.0.to_string()) {
                return Err(errors::Error::ServiceKindUninitialized(t.0.clone()).into());
            }
        }

        Ok(())
    }

    fn has_equal_execution_modes(&self) -> bool {
        let modes: Vec<ServiceExecutionMode> = self.servers.values().map(|s|s.mode()).collect();
        modes.iter().all(|m| *m == modes[0])
    }

    async fn start_features(&mut self) -> merrors::Result<()> {
        self.logger.info("starting features");
        self.context.initialize_features().await
    }

    async fn initialize_service_internals(&mut self) -> merrors::Result<()> {
        let definitions = self.definitions.clone();
        let envs = self.envs.clone();
        let ctx = self.context.clone();

        for s in &definitions.types {
            let svc = self.get_server(&s.0)?;
            svc.initialize(envs.clone(), definitions.clone())?;
            svc.on_start(&ctx).await?;
        }

        Ok(())
    }

    async fn print_service_resources(&self) {
        let mut info: HashMap<String, serde_json::Value> = HashMap::new();

        for feature in self.context.features.lock().await.iter() {
            info.insert(feature.name().to_string(), feature.info());
        }

        self.logger.infof("service resources", serde_json::Value::Object(info.into_iter().collect()));
    }

    async fn run(&mut self) -> merrors::Result<()> {
        let definitions = self.definitions.clone();
        let context = self.context.clone();
        let shutdown_tx = self.shutdown_tx.clone();
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        for s in &definitions.types {
            let svc = self.get_server(&s.0)?.clone();
            let context = context.clone();
            let mut shutdown_rx = shutdown_tx.subscribe();
            let tx = tx.clone();

            self.logger.infof("service is running", svc.info());

            let handle = task::spawn(async move {
                context.logger_ref().debugf("starting service task", serde_json::json!({
                    "task_name": svc.kind().to_string(),
                }));

                if let Err(e) = svc.run(&context, shutdown_rx.clone()).await {
                    let _ = tx.send(e).await;
                    return;
                }

                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        context.logger_ref().debugf("finishing service task", serde_json::json!({
                            "task_name": svc.kind().to_string(),
                        }));
                    }
                }

                context.logger_ref().debugf("service task finished", serde_json::json!({
                    "task_name": svc.kind().to_string(),
                }));
            });

            self.handlers.push(handle);
        }

        // keep running until ctrl+c
        tokio::select! {
            Some(err) = rx.recv() => {
                self.logger.error(err.to_string().as_str());
                self.stop_service_tasks().await?;
                return Err(err);
            }
            _ = self.wait_finishing_signal() => {
                self.stop_service_tasks().await?;
            }
        }

        Ok(())
    }

    async fn wait_finishing_signal(&self) {
        // If we are here is because we already passed the validation, and since
        // we only execute when execution modes are equal for all servers, it does
        // not matter which one we get.
        let mode = self.servers.values().next().unwrap().mode();

        // Wait for a signal to finish the service.
        if mode == ServiceExecutionMode::Block {
            signal::ctrl_c().await.expect("failed to listen for ctrl-c");
        }
    }

    async fn stop_service_tasks(&mut self) -> merrors::Result<()> {
        let definitions = self.definitions.clone();
        let context = self.context.clone();

        // Call service stop callback so the service can stop itself
        for s in &definitions.types {
            let svc = self.get_server(&s.0)?;
            svc.stop(&context).await;
        }

        // Tells the main tasks to stop
        let _ = self.shutdown_tx.send(());
        self.logger.debug("sending shutdown signal for service tasks");

        for handle in &mut self.handlers {
            let _ = handle.await;
        }

        // Calls the callback to release service resources.
        for s in &definitions.types {
            let svc = self.get_server(&s.0)?;
            svc.on_finish().await?;
        }

        // Cleanup features
        self.context.cleanup_features().await;
        self.logger.info("service stopped");

        Ok(())
    }

    fn get_server(&mut self, kind: &ServiceKind) -> Result<&mut Box<dyn plugin::service::Service>, errors::Error> {
        match self.servers.get_mut(&kind.to_string()) {
            None => Err(errors::Error::ServiceNotFound(kind.to_string()).into()),
            Some(s) => Ok(s),
        }
    }
}