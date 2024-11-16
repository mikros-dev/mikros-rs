use std::collections::HashMap;
use std::sync::{Arc};
use std::convert::Infallible;

use axum::Router;
use futures::lock::Mutex;
use http::{request::Request, response::Response};
use tonic::body::BoxBody;
use tonic::server::NamedService;

use crate::{definition, errors as merrors, plugin};
use crate::http::{ServiceInternalState, ServiceState};
use crate::service::native::{NativeService, Native};
use crate::service::script::{ScriptService, Script};
use crate::service::grpc::Grpc;
use crate::service::http::Http;
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

    /// Initializes the native service type with the required structure
    /// implementing its API.
    pub fn native<S>(mut self, svc: Arc<Mutex<S>>) -> Self
    where
        S: NativeService + 'static
    {
        self.servers.insert(definition::ServiceKind::Native.to_string(), Box::new(Native::new(svc)));
        self
    }

    /// Initializes the script service type with the required structure
    /// implementing its API.
    pub fn script<S>(mut self, svc: Arc<Mutex<S>>) -> Self
    where
        S: ScriptService + 'static
    {
        self.servers.insert(definition::ServiceKind::Script.to_string(), Box::new(Script::new(svc)));
        self
    }

    /// Initializes the gRPC service type with the required structure
    /// implementing its API.
    pub fn grpc<S>(mut self, server: S) -> Self
    where
        S: tonic::codegen::Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
            + NamedService
            + Clone
            + Send
            + Sync
            + 'static,
        S::Future: Send + 'static,
    {
        self.servers.insert(definition::ServiceKind::Grpc.to_string(), Box::new(Grpc::new(server)));
        self
    }

    /// Initializes the gRPC service type with the required structure
    /// implementing its API and another with implementing the Lifecycle
    /// API.
    pub fn grpc_with_lifecycle<S, B>(mut self, server: S, lifecycle: Arc<Mutex<B>>) -> Self
    where
        S: tonic::codegen::Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
            + NamedService
            + Clone
            + Send
            + Sync
            + 'static,
        S::Future: Send + 'static,
        B: Lifecycle + 'static
    {
        self.servers.insert(definition::ServiceKind::Grpc.to_string(), Box::new(Grpc::new_with_lifecycle(server, lifecycle)));
        self
    }

    /// Initializes the HTTP service type with the required structure
    /// implementing the service endpoint handlers.
    pub fn http(mut self, router: Router<Arc<Mutex<ServiceState>>>) -> Self {
        self.servers.insert(definition::ServiceKind::Http.to_string(), Box::new(Http::new(router)));
        self
    }

    /// Initializes the HTTP service type with the required structure
    /// implementing the service endpoint handlers and another with
    /// implementing the Lifecycle API.
    pub fn http_with_lifecycle<L>(mut self, router: Router<Arc<Mutex<ServiceState>>>, lifecycle: Arc<Mutex<L>>) -> Self
    where
        L: Lifecycle + 'static
    {
        self.servers.insert(definition::ServiceKind::Http.to_string(), Box::new(Http::new_with_lifecycle(router, lifecycle)));
        self
    }

    pub fn http_with_state(mut self, router: Router<Arc<Mutex<ServiceState>>>, state: Box<dyn ServiceInternalState>) -> Self {
        self.servers.insert(definition::ServiceKind::Http.to_string(), Box::new(Http::new_with_state(router, state)));
        self
    }

    pub fn http_with_lifecycle_and_state<L>(mut self, router: Router<Arc<Mutex<ServiceState>>>, lifecycle: Arc<Mutex<L>>, state: Box<dyn ServiceInternalState>) -> Self
    where
        L: Lifecycle + 'static,
    {
        self.servers.insert(definition::ServiceKind::Http.to_string(), Box::new(Http::new_with_lifecycle_and_state(router, lifecycle, state)));
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