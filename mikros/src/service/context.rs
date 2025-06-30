use futures::lock::Mutex;
use std::sync::Arc;

use crate::definition::Definitions;
use crate::env::Env;
use crate::service::errors::Error;
use crate::{env, errors, logger, plugin};

/// Context gathers all information and APIs available for services to be used
/// when callbacks are called.
#[derive(Clone)]
pub struct Context {
    logger: Arc<logger::Logger>,
    definitions: Arc<Definitions>,

    pub(crate) envs: Arc<Env>,
    pub(crate) features: Arc<Mutex<Vec<Box<dyn plugin::feature::Feature>>>>,
}

impl Context {
    pub(crate) fn new(
        envs: Arc<Env>,
        logger: Arc<logger::Logger>,
        definitions: Arc<Definitions>,
        features: Vec<Box<dyn plugin::feature::Feature>>,
    ) -> Self {
        Self {
            logger,
            envs,
            definitions,
            features: Arc::new(Mutex::new(features)),
        }
    }

    /// Gives the service access to the current logger, so it can display
    /// messages using the same format.
    pub fn logger(&self) -> Arc<logger::Logger> {
        self.logger.clone()
    }

    /// Gives the service access to a reference of the current logger, so it
    /// can display messages using the same format.
    pub fn logger_ref(&self) -> &logger::Logger {
        &self.logger
    }

    /// Gives the service access to the service environments.
    pub fn env(&self) -> Arc<env::Env> {
        self.envs.clone()
    }

    /// Gives the service access to a reference to the service environments.
    pub fn env_ref(&self) -> &env::Env {
        &self.envs
    }

    /// Gives the service access to the service definitions loaded from the
    /// service.toml file.
    pub fn definitions(&self) -> Arc<Definitions> {
        self.definitions.clone()
    }

    /// Gives the service access to a reference of the service definitions loaded
    /// from the service.toml file.
    pub fn definitions_ref(&self) -> &Definitions {
        &self.definitions
    }

    /// Returns the current service name.
    #[must_use]
    pub fn service_name(&self) -> String {
        self.definitions.name.to_string()
    }

    /// Returns the URL connection string for linking services.
    #[must_use]
    pub fn client_connection_url(&self, client_name: &str) -> String {
        match self.definitions.client(client_name) {
            Some(c) => format!("{}:{}", c.host, c.port),
            None => format!(
                "{}.{}:{}",
                client_name,
                self.envs.coupled_namespace.clone(),
                self.envs.coupled_port
            ),
        }
    }

    /// On success, returns the feature found inside the context.
    pub async fn feature(&self, name: &str) -> errors::Result<Box<dyn plugin::feature::Feature>> {
        match self
            .features
            .lock()
            .await
            .iter()
            .find(|f| f.name() == name)
            .cloned()
        {
            None => {
                let error = Error::FeatureNotFound(name.to_string());
                Err(errors::ServiceError::from_error(
                    self.clone().into(),
                    error.into(),
                ))
            }
            Some(f) => {
                if !f.is_enabled() {
                    let error = Error::FeatureDisabled(name.to_string());
                    return Err(errors::ServiceError::from_error(
                        self.clone().into(),
                        error.into(),
                    ));
                }

                Ok(f)
            }
        }
    }

    pub(crate) async fn initialize_features(&mut self) -> errors::Result<()> {
        for feature in self.features.lock().await.iter_mut() {
            if feature.can_be_initialized(self.definitions.clone(), self.envs.clone())? {
                feature.initialize(self.clone().into()).await?;
            }
        }

        Ok(())
    }

    pub(crate) async fn cleanup_features(&mut self) {
        for feature in self.features.lock().await.iter() {
            if feature.is_enabled() {
                feature.cleanup().await;
            }
        }
    }
}

// TODO: Handle unwrap error here to return a proper tonic RPC error
/// Retrieves the mikros Context from an RPC request argument.
pub fn from_request<B>(request: &tonic::Request<B>) -> Arc<Context>
where
    B: prost::Message,
{
    request.extensions().get::<Arc<Context>>().unwrap().clone()
}

/// A macro to help service coupling using gRPC connections.
#[macro_export]
macro_rules! link_grpc_service {
    ($context:ident, $client:ident, $client_name:expr) => {{
        let url = $context.client_connection_url($client_name);
        match $client::connect(url).await {
            Ok(c) => c,
            Err(e) => {
                return Err(mikros::errors::ServiceError::custom(
                    $context,
                    &e.to_string(),
                ));
            }
        }
    }};
}
