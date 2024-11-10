use std::sync::Arc;

use crate::definition::ServiceDefinitions;
use crate::env::Env;
use crate::{errors as merrors, features, plugin};
use crate::service::builder::ServiceBuilder;

/// Context gathers all information and APIs available for services to be used
/// when callbacks are called.
pub struct Context {
    logger: Arc<logger::Logger>,
    envs: Arc<Env>,
    definitions: Arc<ServiceDefinitions>,
    features: Vec<Box<dyn plugin::feature::Feature>>,
}

impl Context {
    pub(crate) fn new(logger: Arc<logger::Logger>, definitions: Arc<ServiceDefinitions>, builder: &ServiceBuilder) -> merrors::Result<Self> {
        let mut features = features::register_features();
        for f in builder.features.clone() {
            features.push(f);
        }

        Ok(Self {
            logger,
            envs: Env::load(&definitions)?,
            definitions,
            features,
        })
    }

    /// Gives the service access to the current logger, so it can display
    /// messages using the same format.
    pub fn logger(&self) -> &logger::Logger {
        &self.logger
    }

    /// Allows a service to retrieve the current value of an environment
    /// variable defined inside the service definitions file.
    pub fn env(&self, name: &str) -> Option<&String> {
        self.envs.get_defined_env(name)
    }

    /// Returns the current service name.
    pub fn service_name(&self) -> String {
        self.definitions.name.clone()
    }

    /// Returns a previously loaded feature to be used by services.
    pub(crate) fn feature(&self, name: &str) -> Option<&Box<dyn plugin::feature::Feature>> {
        self.features.iter().find(|f| f.name() == name)
    }
}