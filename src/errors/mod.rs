mod macros;

use std::fmt::Formatter;
use std::sync::Arc;

use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde_derive::{Deserialize, Serialize};

use crate::logger::Logger;
use crate::service::context::Context;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub(crate) enum Error {
    Internal(String),
    NotFound,
    InvalidArguments,
    PreconditionFailed(String),
    Rpc(String),
    Custom(String),
    PermissionDenied,
}

impl Error {
    pub(crate) fn description(&self) -> String {
        match self {
            Error::Internal(msg) => msg.to_string(),
            Error::NotFound => "not found".to_string(),
            Error::InvalidArguments => "invalid arguments".to_string(),
            Error::PreconditionFailed(msg) => msg.to_string(),
            Error::Rpc(msg) => msg.to_string(),
            Error::Custom(msg) => msg.to_string(),
            Error::PermissionDenied =>  "no permission to access the service".to_string(),
        }
    }

    fn kind(&self) -> String {
        match self {
            Error::Internal(_) => "InternalError".to_string(),
            Error::NotFound => "NotFoundError".to_string(),
            Error::InvalidArguments => "ValidationError".to_string(),
            Error::PreconditionFailed(_) => "ConditionError".to_string(),
            Error::Rpc(_) => "RPCError".to_string(),
            Error::Custom(_) => "CustomError".to_string(),
            Error::PermissionDenied => "PermissionError".to_string(),
        }
    }
}

// The error that a service must return for another service, either through an
// RPC call (between gRPC client-server communication) or as an HTTP response
// for clients.
#[derive(Deserialize, Serialize)]
pub struct ServiceError {
    // Fields that are always serialized
    code: i32,
    kind: String,

    // Fields that can be hidden.
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    service_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    attributes: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    destination: Option<String>,

    #[serde(skip)]
    logger: Option<Arc<Logger>>,

    #[serde(skip)]
    concealable_attributes: Option<Vec<String>>,
}

impl ServiceError {
    fn new(ctx: Arc<Context>, error: Error) -> Self {
        Self {
            code: 0,
            kind: error.kind(),
            message: Some(error.description()),
            service_name: Some(ctx.service_name()),
            attributes: None,
            destination: None,
            logger: Self::get_logger(ctx.clone()),
            concealable_attributes: ctx.envs.response_fields(),
        }
    }

    fn get_logger(ctx: Arc<Context>) -> Option<Arc<Logger>> {
        let logger = ctx.definitions().log();

        if logger.display_errors.unwrap() {
            return Some(ctx.logger().clone())
        }

        None
    }

    /// Sets that the current error is related to an unexpected internal
    /// service behavior.
    pub fn internal(ctx: Arc<Context>, msg: &str) -> Self {
        Self::new(
            ctx,
            Error::Internal(msg.to_string()),
        )
    }

    /// Sets that the current error is related to some data or resource not
    /// being found by the service.
    pub fn not_found(ctx: Arc<Context>) -> Self {
        Self::new(
            ctx,
            Error::NotFound,
        )
    }

    /// Sets that the current error is related to an argument that didn't
    /// follow validation rules.
    pub fn invalid_arguments(ctx: Arc<Context>, _details: serde_json::Value) -> Self {
        Self::new(
            ctx,
            Error::InvalidArguments
        )
    }

    /// Sets that the current error is related to an internal condition which
    /// wasn't satisfied.
    pub fn precondition_failed(ctx: Arc<Context>, msg: &str) -> Self {
        Self::new(
            ctx,
            Error::PreconditionFailed(msg.to_string()),
        )
    }

    /// Sets that the current error is related to a failed RPC call with
    /// another service.
    pub fn rpc(ctx: Arc<Context>, destination: &str, msg: &str) -> Self {
        let mut error = Self::new(
            ctx,
            Error::Rpc(msg.to_string()),
        );

        error.destination = Some(destination.to_string());
        error
    }

    /// Lets a service set a custom error kind for its own error.
    pub fn custom(ctx: Arc<Context>, msg: &str) -> Self {
        Self::new(
            ctx,
            Error::Custom(msg.to_string()),
        )
    }

    /// Sets that the current error is related to a client trying to access
    /// a resource without having permission to do so.
    pub fn permission_denied(ctx: Arc<Context>) -> Self {
        Self::new(
            ctx,
            Error::PermissionDenied,
        )
    }

    /// Adds a code for the error so the client can map and identify their errors.
    pub fn with_code(mut self, code: i32) -> Self {
        self.code = code;
        self
    }

    /// Adds additional information into the error so they can be displayed for
    /// the client if desired.
    pub fn with_attributes(mut self, attributes: serde_json::Value) -> Self {
        self.attributes = Some(attributes);
        self
    }

    fn merge(a: &mut serde_json::Value, b: serde_json::Value) {
        match (a, b) {
            (a @ &mut serde_json::Value::Object(_), serde_json::Value::Object(b)) => {
                let a = a.as_object_mut().unwrap();
                for (k, v) in b {
                    Self::merge(a.entry(k).or_insert(serde_json::Value::Null), v);
                }
            }
            (a, b) => *a = b,
        }
    }

    fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap_or("could not serialize error message".to_string())
    }

    // Just a helper test function to add fields that should be hidden when
    // serialized. This way we don't need to set environment variable for this
    // operation inside the tests.
    #[cfg(test)]
    fn hide_field(mut self, field: &str) -> Self {
        let mut fields = self.concealable_attributes.unwrap_or(Vec::new());
        fields.push(field.to_string());
        self.concealable_attributes = Some(fields);
        self
    }

    // Translates an Error enum into a ServiceError object.
    pub(crate) fn from_error(ctx: Arc<Context>, error: Error) -> Self {
        Self::new(ctx.clone(), error)
    }
}

impl From<ServiceError> for tonic::Status {
    fn from(error: ServiceError) -> Self {
        // Should we log the message?
        if let Some(logger) = &error.logger {
            let mut error_attributes = serde_json::json!({
                "error.code": error.code,
                "error.kind": error.kind,
            });

            if let Some(defined_attributes) = &error.attributes {
                let mut defined_attributes = defined_attributes.clone();
                ServiceError::merge(&mut defined_attributes, error_attributes);
                error_attributes = defined_attributes;
            }

            let message = error.message.clone();
            logger.errorf(&message.unwrap(), error_attributes)
        }

        let mut error = error;

        // Hide fields according what as defined when the application started.
        // It's worth notice that from now on, we only have information that
        // was serialized.
        if let Some(attributes) = &error.concealable_attributes {
            attributes.iter().for_each(|field| {
                let field = field.to_lowercase();

                if field == "message" {
                    error.message = None
                }

                if field == "service_name" {
                    error.service_name = None
                }

                if field == "attributes" {
                    error.attributes = None;
                }

                if field == "destination" {
                    error.destination = None;
                }
            });
        }

        // Return our error always as an (gRPC) Unknown?
        let message = error.serialize();
        tonic::Status::unknown(message)
    }
}

impl From<tonic::Status> for ServiceError {
    fn from(status: tonic::Status) -> Self {
        let error: ServiceError = serde_json::from_str(status.message()).unwrap();
        error
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let code = match self.kind.as_str() {
            "NotFoundError" => StatusCode::NOT_FOUND,
            "ValidationError" => StatusCode::BAD_REQUEST,
            "ConditionError" => StatusCode::PRECONDITION_FAILED,
            "PermissionError" => StatusCode::FORBIDDEN,
            "RPCError" => StatusCode::INTERNAL_SERVER_ERROR,
            "CustomError" => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (code, self.serialize()).into_response()
    }
}

impl std::error::Error for ServiceError {}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.serialize())
    }
}

impl std::fmt::Debug for ServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.serialize())
    }
}

// This is just a simple conversion for public APIs that deal with Error
// internally but must return a ServiceError for the client.
impl From<Error> for ServiceError {
    fn from(error: Error) -> Self {
        Self {
            code: 0,
            kind: error.kind(),
            message: Some(error.description()),
            service_name: None,
            attributes: None,
            destination: None,
            logger: None,
            concealable_attributes: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger::builder::LoggerBuilder;
    use crate::env::Env;
    use crate::definition::Definitions;

    fn assets_path() -> std::path::PathBuf {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("resources/test");
        p
    }

    fn build_context() -> Arc<Context> {
        let filename = assets_path().join("definitions/service.toml.ok_custom_settings");
        let defs = Definitions::new(filename.to_str(), None).unwrap();
        let env = Env::load(&defs).unwrap();
        let logger = Arc::new(LoggerBuilder::new().build());

        Arc::new(Context::new(env, logger, defs, vec![]).unwrap())
    }

    #[test]
    fn test_complete_service_error() {
        let ctx = build_context();
        let error = ServiceError::rpc(ctx.clone(), "http", "connection failed")
            .with_code(42)
            .with_attributes(serde_json::json!({
                "key": "value"
            }));

        assert_eq!(error.code, 42);
        assert_eq!(error.kind, "RPCError");
        assert_eq!(error.message.unwrap(), "connection failed");
        assert_eq!(error.service_name.unwrap(), "my-service");
        assert_eq!(error.attributes.unwrap(), serde_json::json!({
            "key": "value"
        }));

        assert_eq!(error.destination.unwrap(), "http");
    }

    #[test]
    fn test_service_error_without_message_field() {
        let ctx = build_context();
        let error = ServiceError::rpc(ctx.clone(), "http", "connection failed")
            .with_code(42)
            .with_attributes(serde_json::json!({
                "key": "value"
            }))
            .hide_field("message");

        let grpc_error: tonic::Status = error.into();
        let deserialized: ServiceError = grpc_error.into();

        assert_eq!(deserialized.code, 42);
        assert_eq!(deserialized.kind, "RPCError");
        assert_eq!(deserialized.message.is_none(), true);
        assert_eq!(deserialized.service_name.unwrap(), "my-service");
        assert_eq!(deserialized.attributes.unwrap(), serde_json::json!({
            "key": "value"
        }));

        assert_eq!(deserialized.destination.unwrap(), "http");
    }

    #[test]
    fn test_service_error_without_service_name_field() {
        let ctx = build_context();
        let error = ServiceError::rpc(ctx.clone(), "http", "connection failed")
            .with_code(42)
            .with_attributes(serde_json::json!({
                "key": "value"
            }))
            .hide_field("service_name");

        let grpc_error: tonic::Status = error.into();
        let deserialized: ServiceError = grpc_error.into();

        assert_eq!(deserialized.code, 42);
        assert_eq!(deserialized.kind, "RPCError");
        assert_eq!(deserialized.message.unwrap(), "connection failed");
        assert_eq!(deserialized.service_name.is_none(), true);
        assert_eq!(deserialized.attributes.unwrap(), serde_json::json!({
            "key": "value"
        }));

        assert_eq!(deserialized.destination.unwrap(), "http");
    }

    #[test]
    fn test_service_error_without_attributes_field() {
        let ctx = build_context();
        let error = ServiceError::rpc(ctx.clone(), "http", "connection failed")
            .with_code(42)
            .with_attributes(serde_json::json!({
                "key": "value"
            }))
            .hide_field("attributes");

        let grpc_error: tonic::Status = error.into();
        let deserialized: ServiceError = grpc_error.into();

        assert_eq!(deserialized.code, 42);
        assert_eq!(deserialized.kind, "RPCError");
        assert_eq!(deserialized.message.unwrap(), "connection failed");
        assert_eq!(deserialized.service_name.unwrap(), "my-service");
        assert_eq!(deserialized.attributes.is_none(), true);
        assert_eq!(deserialized.destination.unwrap(), "http");
    }

    #[test]
    fn test_service_error_without_destination_field() {
        let ctx = build_context();
        let error = ServiceError::rpc(ctx.clone(), "http", "connection failed")
            .with_code(42)
            .with_attributes(serde_json::json!({
                "key": "value"
            }))
            .hide_field("destination");

        let grpc_error: tonic::Status = error.into();
        let deserialized: ServiceError = grpc_error.into();

        assert_eq!(deserialized.code, 42);
        assert_eq!(deserialized.kind, "RPCError");
        assert_eq!(deserialized.message.unwrap(), "connection failed");
        assert_eq!(deserialized.service_name.unwrap(), "my-service");
        assert_eq!(deserialized.attributes.unwrap(), serde_json::json!({
            "key": "value"
        }));

        assert_eq!(deserialized.destination.is_none(), true);
    }

    #[test]
    fn test_service_error_without_all_fields() {
        let ctx = build_context();
        let error = ServiceError::rpc(ctx.clone(), "http", "connection failed")
            .with_code(42)
            .with_attributes(serde_json::json!({
                "key": "value"
            }))
            .hide_field("message")
            .hide_field("service_name")
            .hide_field("attributes")
            .hide_field("destination");

        let grpc_error: tonic::Status = error.into();
        let deserialized: ServiceError = grpc_error.into();

        assert_eq!(deserialized.code, 42);
        assert_eq!(deserialized.kind, "RPCError");
        assert_eq!(deserialized.message.is_none(), true);
        assert_eq!(deserialized.service_name.is_none(), true);
        assert_eq!(deserialized.attributes.is_none(), true);
        assert_eq!(deserialized.destination.is_none(), true);
    }

    #[test]
    fn test_create_all_service_error_kind() {
        let ctx = build_context();

        // Internal
        let internal = ServiceError::internal(ctx.clone(), "some internal error");
        assert_eq!(internal.kind, "InternalError".to_string());

        // NotFound
        let not_found = ServiceError::not_found(ctx.clone());
        assert_eq!(not_found.kind, "NotFoundError".to_string());

        // InvalidArguments
        let invalid_arguments = ServiceError::invalid_arguments(ctx.clone(), serde_json::json!({}));
        assert_eq!(invalid_arguments.kind, "ValidationError".to_string());

        // PreconditionFailed
        let precondition_failed = ServiceError::precondition_failed(ctx.clone(), "precondition failed");
        assert_eq!(precondition_failed.kind, "ConditionError".to_string());

        // RPC
        let rpc = ServiceError::rpc(ctx.clone(), "example-http", "connection failed");
        assert_eq!(rpc.kind, "RPCError".to_string());

        // Custom
        let custom = ServiceError::custom(ctx.clone(), "custom error");
        assert_eq!(custom.kind, "CustomError".to_string());

        // PermissionDenied
        let permission_denied = ServiceError::permission_denied(ctx.clone());
        assert_eq!(permission_denied.kind, "PermissionError".to_string());
    }
}