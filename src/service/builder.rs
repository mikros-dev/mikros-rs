use std::any::Any;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use axum::Router;
use futures::lock::Mutex;
use http::{request::Request, response::Response};
use tonic::body::BoxBody;
use tonic::server::NamedService;

use crate::http::ServiceState;
use crate::service::Service;
use crate::service::grpc::Grpc;
use crate::service::http::Http;
use crate::service::lifecycle::Lifecycle;
use crate::service::native::{NativeService, Native};
use crate::service::script::{ScriptService, Script};
use crate::{definition, errors as merrors, plugin};

/// The builder API to build a mikros service instance. It allows to initialize
/// each type of configured service (inside the service.toml file) with its own
/// data type.
pub struct ServiceBuilder {
    pub(crate) servers: HashMap<String, Box<dyn plugin::service::Service>>,
    pub(crate) features: Vec<Box<dyn plugin::feature::Feature>>,
    pub(crate) custom_service_types: Vec<String>,
}

impl ServiceBuilder {
    fn new() -> Self {
        Self {
            servers: HashMap::new(),
            features: Vec::new(),
            custom_service_types: Vec::new(),
        }
    }

    fn abort(err: merrors::Error) {
        panic!("{}", err.to_string())
    }

    /// Initializes the native service type with the required structure implementing
    /// its API.
    pub fn native(mut self, svc: Box<dyn NativeService>) -> Self {
        let kind = definition::ServiceKind::Native;

        if self.servers.contains_key(&kind.to_string()) {
            Self::abort(merrors::Error::BuilderFailed(format!("{} service already initialized", kind)));
        }

        self.servers.insert(kind.to_string(), Box::new(Native::new(svc)));
        self
    }

    /// Initializes the script service type with the required structure implementing
    /// its API.
    pub fn script(mut self, svc: Box<dyn ScriptService>) -> Self {
        let kind = definition::ServiceKind::Script;

        if self.servers.contains_key(&kind.to_string()) {
            Self::abort(merrors::Error::BuilderFailed(format!("{} service already initialized", kind)));
        }

        self.servers.insert(kind.to_string(), Box::new(Script::new(svc)));
        self
    }

    /// Initializes the gRPC service type with the required structure implementing
    /// its API.
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
        let kind = definition::ServiceKind::Grpc;

        if self.servers.contains_key(&kind.to_string()) {
            Self::abort(merrors::Error::BuilderFailed(format!("{} service already initialized", kind)));
        }

        self.servers.insert(kind.to_string(), Box::new(Grpc::new(server)));
        self
    }

    /// Initializes the gRPC service type with the required structure implementing
    /// its API and another with implementing the Lifecycle API.
    pub fn grpc_with_lifecycle<S, L>(mut self, server: S, lifecycle: Arc<Mutex<L>>) -> Self
    where
        S: tonic::codegen::Service<Request<BoxBody>, Response = Response<BoxBody>, Error = Infallible>
            + NamedService
            + Clone
            + Send
            + Sync
            + 'static,
        S::Future: Send + 'static,
        L: Lifecycle + 'static
    {
        let kind = definition::ServiceKind::Grpc;

        if self.servers.contains_key(&kind.to_string()) {
            Self::abort(merrors::Error::BuilderFailed(format!("{} service already initialized", kind)));
        }

        self.servers.insert(kind.to_string(), Box::new(Grpc::new_with_lifecycle(server, lifecycle)));
        self
    }

    /// Initializes the HTTP service type with the required structure implementing
    /// the service endpoint handlers.
    pub fn http(mut self, router: Router<Arc<Mutex<ServiceState>>>) -> Self {
        let kind = definition::ServiceKind::Http;

        if self.servers.contains_key(&kind.to_string()) {
            Self::abort(merrors::Error::BuilderFailed(format!("{} service already initialized", kind)));
        }

        self.servers.insert(kind.to_string(), Box::new(Http::new(router)));
        self
    }

    /// Initializes the HTTP service type with the required structure implementing
    /// the service endpoint handlers and another with implementing the Lifecycle
    /// API.
    pub fn http_with_lifecycle<L>(
        mut self,
        router: Router<Arc<Mutex<ServiceState>>>,
        lifecycle: Arc<Mutex<L>>,
    ) -> Self
    where
        L: Lifecycle + 'static
    {
        let kind = definition::ServiceKind::Http;

        if self.servers.contains_key(&kind.to_string()) {
            Self::abort(merrors::Error::BuilderFailed(format!("{} service already initialized", kind)));
        }

        self.servers.insert(kind.to_string(), Box::new(Http::new_with_lifecycle(router, lifecycle)));
        self
    }

    /// Initializes the HTTP service type with the required structure implementing
    /// the service endpoint handlers. It also receives an object that will be
    /// passed inside the handlers state.
    pub fn http_with_state<S>(
        mut self,
        router: Router<Arc<Mutex<ServiceState>>>,
        state: Arc<Mutex<S>>,
    ) -> Self
    where
        S: Any + Send + Sync
    {
        let kind = definition::ServiceKind::Http;

        if self.servers.contains_key(&kind.to_string()) {
            Self::abort(merrors::Error::BuilderFailed(format!("{} service already initialized", kind)));
        }

        self.servers.insert(kind.to_string(), Box::new(Http::new_with_state(router, state)));
        self
    }

    /// Initializes the HTTP service type with the required structure implementing
    /// the service endpoint handlers and another with implementing the Lifecycle
    /// API. It also receives an object that will be passed inside the handlers
    /// state.
    pub fn http_with_lifecycle_and_state<L, S>(
        mut self,
        router: Router<Arc<Mutex<ServiceState>>>,
        lifecycle: Arc<Mutex<L>>,
        state: Arc<Mutex<S>>,
    ) -> Self
    where
        L: Lifecycle + 'static,
        S: Any + Send + Sync
    {
        let kind = definition::ServiceKind::Http;

        if self.servers.contains_key(&kind.to_string()) {
            Self::abort(merrors::Error::BuilderFailed(format!("{} service already initialized", kind)));
        }

        self.servers.insert(kind.to_string(), Box::new(Http::new_with_lifecycle_and_state(router, lifecycle, state)));
        self
    }

    /// Adds external features into the current service environment so they can
    /// be used inside the proper service.
    pub fn with_features(mut self, features: Vec<Box<dyn plugin::feature::Feature>>) -> Self {
        self.features.extend(features);
        self
    }

    /// Initializes the service as a custom one. The user must provide here the
    /// proper service implementation with its object implementing its API.
    pub fn custom(mut self, custom_service: Box<dyn plugin::service::Service>) -> Self {
        let service_type = custom_service.kind().to_string();

        if self.servers.contains_key(&service_type) {
            Self::abort(merrors::Error::BuilderFailed(format!("{} service already initialized", service_type)));
        }

        self.servers.insert(service_type.clone(), custom_service);
        self.custom_service_types.push(service_type);

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