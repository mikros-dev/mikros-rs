pub mod builder;
pub mod context;
pub mod grpc;
pub mod native;
pub mod script;

mod lifecycle;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::watch;
use tokio::task::{self, JoinHandle};

use crate::args::Args;
use crate::definition::{Definitions, ServiceKind};
use crate::{errors as merrors, plugin};
use crate::env::Env;
use crate::service::builder::ServiceBuilder;

pub struct Service {
    envs: Arc<Env>,
    definitions: Arc<Definitions>,
    logger: Arc<logger::Logger>,
    servers: HashMap<String, Box<dyn plugin::service::Service>>,
    context: context::Context,
    handles: Vec<JoinHandle<()>>,
    shutdown_tx: watch::Sender<()>,
}

impl Service {
    pub fn new(builder: &ServiceBuilder) -> merrors::Result<Self> {
        let definitions = Service::load_definitions(builder)?;
        let logger = Self::start_logger(&definitions);
        let (shutdown_tx, _) = watch::channel(());
        let envs = Env::load(&definitions)?;

        Ok(Service {
            envs: envs.clone(),
            definitions: definitions.clone(),
            logger: logger.clone(),
            context: Self::build_context(envs.clone(), logger, definitions, builder)?,
            servers: builder.servers.clone(),
            handles: Vec::new(),
            shutdown_tx,
        })
    }

    fn load_definitions(_builder: &ServiceBuilder) -> merrors::Result<Arc<Definitions>> {
        let args = Args::load();

        // FIXME: set custom_info from features
        Definitions::new(args.config_path.as_deref(), None)
    }

    fn start_logger(defs: &Definitions) -> Arc<logger::Logger> {
        Arc::new(logger::builder::LoggerBuilder::default()
            .with_field("svc.name", logger::fields::FieldValue::String(defs.name.clone()))
            .with_field("svc.version", logger::fields::FieldValue::String(defs.version.clone()))
            .with_field("svc.product", logger::fields::FieldValue::String(defs.product.clone()))
            .with_field("svc.language", logger::fields::FieldValue::String(defs.language.clone()))
            .build())
    }

    fn build_context(
        envs: Arc<Env>,
        logger: Arc<logger::Logger>,
        defs: Arc<Definitions>,
        builder: &ServiceBuilder,
    ) -> merrors::Result<context::Context> {
        context::Context::new(envs, logger, defs, builder)
    }

    /// Puts the service to run.
    pub async fn start(&mut self) -> merrors::Result<()> {
        self.logger.info("service starting");
        self.validate_definitions()?;
        self.start_features()?;
        self.initialize_service_internals()?;
        self.print_service_resources();
        self.run().await
    }

    fn validate_definitions(&self) -> merrors::Result<()> {
        if self.servers.is_empty() {
            return Err(merrors::Error::EmptyServiceFound)
        }

        // Script services should be a single service.
        if self.definitions.is_script_service() && self.definitions.types.len() > 1 {
            return Err(merrors::Error::UnsupportedServicesCombination)
        }

        for t in &self.definitions.types {
            if let None = self.servers.get(&t.0.to_string()) {
                return Err(merrors::Error::ServiceKindUninitialized(t.0.clone()))
            }
        }

        Ok(())
    }

    fn start_features(&self) -> merrors::Result<()> {
        // TODO
        Ok(())
    }

    fn initialize_service_internals(&self) -> merrors::Result<()> {
        for s in &self.definitions.types {
            let mut svc = self.get_server(&s.0)?.clone();
            svc.initialize(self.envs.clone(), self.definitions.clone())?
        }

        // couple clients
        // call lifecycle on_start

        Ok(())
    }

    fn print_service_resources(&self) {
        // TODO
    }

    async fn run(&mut self) -> merrors::Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        for s in &self.definitions.types {
            let mut svc = self.get_server(&s.0)?.clone();
            let context = self.context.clone();
            let mut shutdown_rx = self.shutdown_tx.subscribe();
            let tx = tx.clone();

            self.logger.infof("service is running", svc.info());

            let handle = task::spawn(async move {
                context.logger().debugf("starting service task", logger::fields![
                    "task_name" => logger::fields::FieldValue::String(svc.kind().to_string()),
                ]);

                loop {
                    if let Err(e) = svc.run(&context).await {
                        let _ = tx.send(e).await;
                        return;
                    }

                    tokio::select! {
                        _ = shutdown_rx.changed() => {
                            context.logger().debugf("finishing service task", logger::fields![
                                "task_name" => logger::fields::FieldValue::String(svc.kind().to_string()),
                            ]);

                            break;
                        }
                    }
                }

                context.logger().debugf("service task finished",logger::fields![
                    "task_name" => logger::fields::FieldValue::String(svc.kind().to_string()),
                ]);
            });

            self.handles.push(handle);
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
        // Wait for a signal to finish the service.
        if !self.definitions.is_script_service() {
            signal::ctrl_c().await.expect("failed to listen for ctrl-c");
        }
    }

    async fn stop_service_tasks(&mut self) -> merrors::Result<()> {
        // Call service stop callback so the service can stop itself
        for s in &self.definitions.types {
            let mut svc = self.get_server(&s.0)?.clone();
            svc.stop(&self.context).await;
        }

        // Then stops our task runner
        let _ = self.shutdown_tx.send(());
        self.logger.debug("sending shutdown signal for service tasks");

        for handle in &mut self.handles {
            let _ = handle.await;
        }

        self.logger.info("service stopped");
        Ok(())
    }

    fn get_server(&self, kind: &ServiceKind) -> merrors::Result<&Box<dyn plugin::service::Service>> {
        match self.servers.get(&kind.to_string()) {
            None => Err(merrors::Error::NotFound(format!("service {} implementation not found", kind))),
            Some(s) => Ok(s),
        }
    }
}