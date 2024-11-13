use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{definition, errors as merrors, plugin};
use crate::service::native::{NativeService, Native};
use crate::service::script::{ScriptService, Script};
use crate::service::grpc::Grpc;
use crate::service::Service;

pub struct ServiceBuilder {
    pub(crate) servers: HashMap<String, Box<dyn plugin::service::Service>>,
    pub(crate) features: Vec<Box<dyn plugin::feature::Feature>>,
    pub(crate) services: Vec<Box<dyn plugin::service::Service>>,
}

impl ServiceBuilder {
    fn new() -> Self {
        Self {
            servers: HashMap::new(),
            features: Vec::new(),
            services: Vec::new(),
        }
    }

    /// Sets the current service as a native type.
    pub fn as_native<S: NativeService + 'static>(&mut self, svc: Arc<Mutex<S>>) -> &mut Self {
        self.servers.insert(definition::ServiceKind::Native.to_string(), Box::new(Native::new(svc)));
        self
    }

    /// Sets the current service as a script type.
    pub fn as_script<S: ScriptService + 'static>(&mut self, svc: Arc<Mutex<S>>) -> &mut Self {
        self.servers.insert(definition::ServiceKind::Script.to_string(), Box::new(Script::new(svc)));
        self
    }

    pub fn as_grpc(&mut self) -> &mut Self {
        self.servers.insert(definition::ServiceKind::Grpc.to_string(), Box::new(Grpc::new()));
        self
    }

    /// Adds external features into the current service environment.
    pub fn with_features(&mut self, features: Vec<Box<dyn plugin::feature::Feature>>) -> &mut Self {
        self.features.extend(features);
        self
    }

    /// Adds external service type implementations into the current service
    /// environment.
    pub fn with_services(&mut self, services: Vec<Box<dyn plugin::service::Service>>) -> &mut Self {
        self.services.extend(services);
        self
    }

    /// Builds the service to be executed.
    pub fn build(&mut self) -> merrors::Result<Service> {
        Service::new(self)
    }
}

impl Default for ServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}