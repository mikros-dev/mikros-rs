use std::collections::HashMap;
use std::sync::{Arc};
use std::convert::Infallible;

use futures::lock::Mutex;
use http::{request::Request, response::Response};
use tonic::body::BoxBody;
use tonic::server::NamedService;

use crate::{definition, errors as merrors, plugin};
use crate::service::native::{NativeService, Native};
use crate::service::script::{ScriptService, Script};
use crate::service::grpc::Grpc;
use crate::service::lifecycle::Lifecycle;
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
    pub fn native<S: NativeService + 'static>(mut self, svc: Arc<Mutex<S>>) -> Self {
        self.servers.insert(definition::ServiceKind::Native.to_string(), Box::new(Native::new(svc)));
        self
    }

    /// Sets the current service as a script type.
    pub fn script<S: ScriptService + 'static>(mut self, svc: Arc<Mutex<S>>) -> Self {
        self.servers.insert(definition::ServiceKind::Script.to_string(), Box::new(Script::new(svc)));
        self
    }

    pub fn grpc<S>(mut self, svc: S) -> Self
    where
        S: tonic::codegen::Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
            + NamedService
            + Lifecycle
            + Clone
            + Send
            + Sync
            + 'static,
        S::Future: Send + 'static,
    {
        self.servers.insert(definition::ServiceKind::Grpc.to_string(), Box::new(Grpc::new(svc)));
        self
    }

    /// Adds external features into the current service environment.
    pub fn with_features(mut self, features: Vec<Box<dyn plugin::feature::Feature>>) -> Self {
        self.features.extend(features);
        self
    }

    /// Adds external service type implementations into the current service
    /// environment.
    pub fn with_services(mut self, services: Vec<Box<dyn plugin::service::Service>>) -> Self {
        self.services.extend(services);
        self
    }

    /// Builds the service to be executed.
    pub fn build(self) -> merrors::Result<Service> {
        Service::new(self)
    }
}

impl Default for ServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}