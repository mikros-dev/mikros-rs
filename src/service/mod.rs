pub mod builder;
pub mod context;
pub mod native;
pub mod script;

mod lifecycle;

use crate::args::Args;
use crate::definition::ServiceDefinitions;
use crate::{errors as merrors, plugin};
use crate::service::builder::ServiceBuilder;

pub struct Service {
    definitions: std::sync::Arc<ServiceDefinitions>,
    logger: std::sync::Arc<logger::Logger>,
    servers: Vec<Box<dyn plugin::service::Service>>,
    context: context::Context,
}

impl Service {
    pub fn new(builder: &ServiceBuilder) -> merrors::Result<Self> {
        let definitions = Service::load_definitions(builder)?;
        let logger = Self::start_logger(&definitions);

        Ok(Service {
            definitions: definitions.clone(),
            logger: logger.clone(),
            context: Self::build_context(logger, definitions, builder)?,
            servers: builder.servers.clone(),
        })
    }

    fn load_definitions(_builder: &ServiceBuilder) -> merrors::Result<std::sync::Arc<ServiceDefinitions>> {
        let args = Args::load();

        // FIXME: set custom_info from features
        ServiceDefinitions::new(args.config_path.as_deref(), None)
    }

    fn start_logger(defs: &ServiceDefinitions) -> std::sync::Arc<logger::Logger> {
        std::sync::Arc::new(logger::builder::LoggerBuilder::default()
            .with_field("svc.name", logger::fields::FieldValue::String(defs.name.clone()))
            .with_field("svc.version", logger::fields::FieldValue::String(defs.version.clone()))
            .with_field("svc.product", logger::fields::FieldValue::String(defs.product.clone()))
            .with_field("svc.language", logger::fields::FieldValue::String(defs.language.clone()))
            .build())
    }

    fn build_context(
        logger: std::sync::Arc<logger::Logger>,
        defs: std::sync::Arc<ServiceDefinitions>,
        builder: &ServiceBuilder,
    ) -> merrors::Result<context::Context> {
        context::Context::new(logger.clone(), defs, builder)
    }

    /// Puts the service to run.
    pub fn start(&mut self) -> merrors::Result<()> {
        if self.servers.is_empty() {
            return Err(merrors::Error::EmptyServiceFound)
        }

        self.logger.info("service starting");

        // execute service main tasks (in background)
        for s in self.servers.iter_mut() {
            if self.definitions.is_service_configured(s.name()) {
                s.run(&self.context)?;
            }
        }

        // keep running until ctrl+c
        // execute library drop and finish service

        Ok(())
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        for s in &mut self.servers {
            if self.definitions.is_service_configured(s.name()) {
                s.stop(&self.context);
            }
        }

        self.logger.info("service stopped");
    }
}