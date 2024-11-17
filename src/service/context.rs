use std::sync::{Arc, Mutex};

use crate::definition::Definitions;
use crate::env::Env;
use crate::{errors as merrors, plugin};

/// Context gathers all information and APIs available for services to be used
/// when callbacks are called.
#[derive(Clone)]
pub struct Context {
    logger: Arc<logger::Logger>,
    envs: Arc<Env>,
    definitions: Arc<Definitions>,

    pub(crate) features: Arc<Mutex<Vec<Box<dyn plugin::feature::Feature>>>>
}

impl Context {
    pub(crate) fn new(
        envs: Arc<Env>,
        logger: Arc<logger::Logger>,
        definitions: Arc<Definitions>,
        features: Vec<Box<dyn plugin::feature::Feature>>,
    ) -> merrors::Result<Self> {
        Ok(Self {
            logger,
            envs,
            definitions,
            features: Arc::new(Mutex::new(features)),
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
}

/// Retrieves the mikros Context from an RPC request argument.
pub fn from_request<B>(request: &tonic::Request<B>) -> Arc<Context>
where
    B: prost::Message,
{
    request.extensions().get::<Arc<Context>>().unwrap().clone()
}
