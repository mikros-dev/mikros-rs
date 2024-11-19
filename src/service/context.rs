use std::sync::Arc;

use futures::lock::Mutex;

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

    pub(crate) features: Arc<Mutex<Vec<Box<dyn plugin::feature::Feature>>>>,
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
    pub fn logger(&self) -> Arc<logger::Logger> {
        self.logger.clone()
    }

    /// Gives the service access to a reference from the current logger, so it
    /// can display messages using the same format.
    pub fn logger_ref(&self) -> &logger::Logger {
        &self.logger
    }

    /// Allows a service to retrieve the current value of an environment
    /// variable defined inside the service definitions file.
    pub fn env(&self, name: &str) -> Option<&String> {
        self.envs.get_defined_env(name)
    }

    pub fn definitions(&self) -> Arc<Definitions> {
        self.definitions.clone()
    }

    pub fn definitions_ref(&self) -> &Definitions {
        &self.definitions
    }

    /// Returns the current service name.
    pub fn service_name(&self) -> String {
        self.definitions.name.clone()
    }

    pub async fn feature(&self, name: &str) -> Option<Box<dyn plugin::feature::Feature>> {
        self.features.lock().await.iter().find(|f| f.name() == name).cloned()
    }

    pub(crate) async fn initialize_features(&mut self) -> merrors::Result<()> {
        for feature in self.features.lock().await.iter_mut() {
            if feature.can_be_initialized(self.definitions.clone(), self.envs.clone())? {
                feature.initialize(self).await?;
            }
        }

        Ok(())
    }

    pub(crate) async fn cleanup_features(&mut self) {
        for feature in self.features.lock().await.iter() {
            if feature.is_enabled() {
                feature.cleanup().await
            }
        }
    }
}

/// Retrieves the mikros Context from an RPC request argument.
pub fn from_request<B>(request: &tonic::Request<B>) -> Arc<Context>
where
    B: prost::Message,
{
    request.extensions().get::<Arc<Context>>().unwrap().clone()
}
