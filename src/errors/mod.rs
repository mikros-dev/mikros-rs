use std::fmt::Formatter;
use std::sync::Arc;

use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde_derive::{Deserialize, Serialize};
use serde_json::json;

use crate::logger::Logger;
use crate::service::context::Context;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Deserialize, Serialize)]
pub enum Error {
    Internal(String),
    NotFound,
    InvalidArguments,
    PreconditionFailed(String),
    RPC(String),
    Custom(String),
    PermissionDenied,
}

impl Error {
    fn description(&self) -> String {
        match self {
            Error::Internal(msg) => msg.to_string(),
            Error::NotFound => "not found".to_string(),
            Error::InvalidArguments => "invalid arguments".to_string(),
            Error::PreconditionFailed(msg) => msg.to_string(),
            Error::RPC(msg) => msg.to_string(),
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
            Error::RPC(_) => "RPCError".to_string(),
            Error::Custom(_) => "CustomError".to_string(),
            Error::PermissionDenied => "PermissionError".to_string(),
        }
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
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
            message: Some(error.description()),
            kind: error.kind(),
            service_name: Some(ctx.service_name()),
            attributes: None,
            logger: Self::get_logger(ctx.clone()),
            destination: None,
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
            Error::RPC(msg.to_string()),
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
}

impl From<ServiceError> for tonic::Status {
    fn from(error: ServiceError) -> Self {
        // Should we log the message?
        if let Some(logger) = &error.logger {
            let error_attributes = json!({
                "error.code": error.code,
                "error.kind": error.kind,
            });

            let attributes = if let Some(defined_attributes) = &error.attributes {
                let mut defined_attributes = defined_attributes.clone();
                ServiceError::merge(&mut defined_attributes, error_attributes);
                defined_attributes.clone()
            } else {
                error_attributes
            };

            let message = error.message.clone();
            logger.errorf(&message.unwrap(), attributes)
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

// A macro to ease internal modules error creation with common behavior
// implemented. It creates an enum $name with all $entry defined. All
// errors created from this enum are returned as Internal for the client.
//
// The entries can have 0 or more values in addition to your description
// message.
//
// It can be called like this:
// internal_error!(
//      Error {
//          Internal(msg: String) => "Internal error {}",
//          NotFound => "Not found"
//      }
// )
//
// And an enum like this will be declared:
//
// enum Error {
//      Internal(String),
//      NotFound,
// }
#[macro_export]
macro_rules! internal_errors {
      ($name:ident { $($entry:ident$(($($arg:ident : $arg_type:ty),*))? => $desc:expr),* }) => {
        pub enum $name {
            $(
                $entry$(($($arg_type),*))?,
            )*
        }

        impl $name {
            pub fn description(&self) -> String {
                match self {
                    $(
                        $name::$entry$(($($arg),*))? => format!($desc, $($($arg),*)?),
                    )*
                }
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.description())
            }
        }

        impl From<$name> for $crate::errors::Error {
            fn from(e: $name) -> $crate::errors::Error {
                $crate::errors::Error::Internal(e.description())
            }
        }
    };
}