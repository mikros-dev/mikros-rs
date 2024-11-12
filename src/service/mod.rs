pub mod builder;
pub mod context;
pub mod grpc;
pub mod native;
pub mod script;

mod lifecycle;

use tokio::signal;
use tokio::sync::watch;
use tokio::task::{self, JoinHandle};

use crate::args::Args;
use crate::definition::Definitions;
use crate::{errors as merrors, plugin};
use crate::service::builder::ServiceBuilder;

pub struct Service {
    definitions: std::sync::Arc<Definitions>,
    logger: std::sync::Arc<logger::Logger>,
    servers: Vec<Box<dyn plugin::service::Service>>,
    context: context::Context,
    handles: Vec<JoinHandle<()>>,
    shutdown_tx: watch::Sender<()>,
}

impl Service {
    pub fn new(builder: &ServiceBuilder) -> merrors::Result<Self> {
        let definitions = Service::load_definitions(builder)?;
        let logger = Self::start_logger(&definitions);
        let (shutdown_tx, _) = watch::channel(());

        Ok(Service {
            definitions: definitions.clone(),
            logger: logger.clone(),
            context: Self::build_context(logger, definitions, builder)?,
            servers: builder.servers.clone(),
            handles: Vec::new(),
            shutdown_tx,
        })
    }

    fn load_definitions(_builder: &ServiceBuilder) -> merrors::Result<std::sync::Arc<Definitions>> {
        let args = Args::load();

        // FIXME: set custom_info from features
        Definitions::new(args.config_path.as_deref(), None)
    }

    fn start_logger(defs: &Definitions) -> std::sync::Arc<logger::Logger> {
        std::sync::Arc::new(logger::builder::LoggerBuilder::default()
            .with_field("svc.name", logger::fields::FieldValue::String(defs.name.clone()))
            .with_field("svc.version", logger::fields::FieldValue::String(defs.version.clone()))
            .with_field("svc.product", logger::fields::FieldValue::String(defs.product.clone()))
            .with_field("svc.language", logger::fields::FieldValue::String(defs.language.clone()))
            .build())
    }

    fn build_context(
        logger: std::sync::Arc<logger::Logger>,
        defs: std::sync::Arc<Definitions>,
        builder: &ServiceBuilder,
    ) -> merrors::Result<context::Context> {
        context::Context::new(logger.clone(), defs, builder)
    }

    /// Puts the service to run.
    pub async fn start(&mut self) -> merrors::Result<()> {
        self.start_service_validations()?;
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        // execute service main task
        for s in self.servers.iter_mut() {
            self.logger.infof("service starting", s.information());

            if self.definitions.is_service_configured(s.name()) {
                let mut service = s.clone();
                let context = self.context.clone();
                let mut shutdown_rx = self.shutdown_tx.subscribe();
                let tx = tx.clone();

                let handle = task::spawn(async move {
                    context.logger().debugf("starting service task", logger::fields![
                        "task_name" => logger::fields::FieldValue::String(service.name().to_string()),
                    ]);

                    loop {
                        if let Err(e) = service.run(&context).await {
                            let _ = tx.send(e).await;
                            return;
                        }

                        tokio::select! {
                            _ = shutdown_rx.changed() => {
                                context.logger().debugf("finishing service task", logger::fields![
                                    "task_name" => logger::fields::FieldValue::String(service.name().to_string()),
                                ]);

                                break;
                            }
                        }
                    }

                    context.logger().debugf("service task finished",logger::fields![
                        "task_name" => logger::fields::FieldValue::String(service.name().to_string()),
                    ]);
                });

                self.handles.push(handle);
            }
        }

        // keep running until ctrl+c
        tokio::select! {
            Some(err) = rx.recv() => {
                self.logger.error(err.to_string().as_str());
                self.stop_service_tasks().await;
                return Err(err);
            }
            _ = self.wait_finishing_signal() => {
                self.stop_service_tasks().await;
            }
        }

        Ok(())
    }

    fn start_service_validations(&mut self) -> merrors::Result<()> {
        if self.servers.is_empty() {
            return Err(merrors::Error::EmptyServiceFound)
        }

        // Script services should be a single service.
        if self.definitions.is_script_service() && self.definitions.types.len() > 1 {
            return Err(merrors::Error::UnsupportedServicesCombination)
        }

        Ok(())
    }

    async fn wait_finishing_signal(&self) {
        // Wait for a signal to finish the service.
        if !self.definitions.is_script_service() {
            signal::ctrl_c().await.expect("failed to listen for ctrl-c");
        }
    }

    async fn stop_service_tasks(&mut self) {
        // Call service stop callback so the service can stop itself
        for s in &mut self.servers {
            if self.definitions.is_service_configured(s.name()) {
                s.stop(&self.context).await;
            }
        }

        // Then stops our task runner
        let _ = self.shutdown_tx.send(());
        self.logger.debug("sending shutdown signal for service tasks");

        for handle in &mut self.handles {
            let _ = handle.await;
        }

        self.logger.info("service stopped");
    }
}