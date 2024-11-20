pub mod builder;
pub mod context;
pub mod grpc;
pub mod http;
pub mod lifecycle;
pub mod native;
pub mod script;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::watch;
use tokio::task::{self, JoinHandle};

use crate::args::Args;
use crate::definition::{Definitions, ServiceKind, CustomServiceInfo};
use crate::{errors as merrors, features, plugin};
use crate::env::Env;
use crate::plugin::service::ServiceExecutionMode;
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
    pub fn new(builder: ServiceBuilder) -> merrors::Result<Self> {
        let definitions = Service::load_definitions(&builder)?;
        let logger = Self::start_logger(&definitions);
        let (shutdown_tx, _) = watch::channel(());
        let envs = Env::load(&definitions)?;

        let mut features = features::register_features();
        for f in builder.features.clone() {
            features.push(f);
        }

        Ok(Service {
            envs: envs.clone(),
            definitions: definitions.clone(),
            logger: logger.clone(),
            context: Self::build_context(envs.clone(), logger, definitions, features)?,
            servers: builder.servers,
            handles: Vec::new(),
            shutdown_tx,
        })
    }

    fn load_definitions(builder: &ServiceBuilder) -> merrors::Result<Arc<Definitions>> {
        let args = Args::load();
        let custom_info = if !builder.custom_service_types.is_empty() {
            Some(CustomServiceInfo{
                //types: Some(external_service_types),
                types: Some(builder.custom_service_types.clone()),
            })
        } else {
            None
        };

        Definitions::new(args.config_path.as_deref(), custom_info)
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
        features: Vec<Box<dyn plugin::feature::Feature>>,
    ) -> merrors::Result<context::Context> {
        context::Context::new(envs, logger, defs, features)
    }

    /// Puts the service to run.
    pub async fn start(&mut self) {
        self.logger.info("service starting");

        if let Err(e) = self.start_service().await {
            self.logger.error(&e.to_string());
            std::process::exit(1);
        }
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
            return Err(merrors::Error::EmptyServiceFound)
        }

        // Script services should be a single service.
        if !self.has_equal_execution_modes() {
            return Err(merrors::Error::UnsupportedServicesExecutionMode)
        }

        for t in &self.definitions.types {
            if !self.servers.contains_key(&t.0.to_string()) {
                return Err(merrors::Error::ServiceKindUninitialized(t.0.clone()))
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

        for s in &definitions.types {
            let svc = self.get_server(&s.0)?;
            svc.initialize(envs.clone(), definitions.clone())?
        }

        // TODO couple clients

        for s in &definitions.types {
            let svc = self.get_server(&s.0)?;
            svc.on_start().await?;
        }

        Ok(())
    }

    async fn print_service_resources(&self) {
        let mut info: HashMap<String, logger::fields::FieldValue> = HashMap::new();

        for feature in self.context.features.lock().await.iter() {
            let i = feature.info();
            info.extend(i.iter().map(|(k, v)| (k.clone(), v.clone())));
        }

        self.logger.infof("service resources", info);
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
                context.logger_ref().debugf("starting service task", logger::fields![
                    "task_name" => logger::fields::FieldValue::String(svc.kind().to_string()),
                ]);

                if let Err(e) = svc.run(&context, shutdown_rx.clone()).await {
                    let _ = tx.send(e).await;
                    return;
                }

                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        context.logger_ref().debugf("finishing service task", logger::fields![
                            "task_name" => logger::fields::FieldValue::String(svc.kind().to_string()),
                        ]);

                    }
                }

                context.logger_ref().debugf("service task finished", logger::fields![
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
        // If we are here is because we already passed the validation and since
        // we only execute when execution modes are equal of all servers, it does
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

        for handle in &mut self.handles {
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

    fn get_server(&mut self, kind: &ServiceKind) -> merrors::Result<&mut Box<dyn plugin::service::Service>> {
        match self.servers.get_mut(&kind.to_string()) {
            None => Err(merrors::Error::NotFound(format!("service {} implementation not found", kind))),
            Some(s) => Ok(s),
        }
    }
}