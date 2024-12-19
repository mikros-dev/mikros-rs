use std::fmt::Formatter;
use std::sync::Arc;

use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde_derive::{Deserialize, Serialize};

use crate::logger::Logger;
use crate::service::context::Context;

pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    Internal(String),
    NotFound(String),
    InvalidArguments(String),
    PreconditionFailed(String),
    RPC(String, String),
    Custom(String),
    PermissionDenied(String),
}

impl Error {
    fn description(&self) -> String {
        match self {
            Error::Internal(msg) => format!("internal error: {}", msg),
            Error::NotFound(s) => format!("not found: {}", s),
            Error::InvalidArguments(s) => format!("invalid arguments: {}", s),
            Error::PreconditionFailed(s) => format!("precondition failed: {}", s),
            Error::RPC(to, msg) => format!("RPC to '{}' failed: {}", to, msg),
            Error::Custom(msg) => format!("service error: {}", msg),
            Error::PermissionDenied(msg) => format!("permission denied: {}", msg),
        }
    }

    fn kind(&self) -> String {
        match self {
            Error::Internal(_) => "InternalError".to_string(),
            Error::NotFound(_) => "NotFoundError".to_string(),
            Error::InvalidArguments(_) => "ValidationError".to_string(),
            Error::PreconditionFailed(_) => "ConditionError".to_string(),
            Error::RPC(_, _) => "RPCError".to_string(),
            Error::Custom(_) => "CustomError".to_string(),
            Error::PermissionDenied(_) => "PermissionError".to_string(),
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
    code: i32,
    message: String,
    kind: String,
    service_name: String,
    attributes: Option<serde_json::Value>,

    #[serde(skip)]
    logger: Option<Arc<Logger>>,
}

impl ServiceError {
    fn new(ctx: Arc<Context>, error: Error) -> Self {
        Self {
            code: 0,
            message: error.description(),
            kind: error.kind(),
            service_name: ctx.service_name(),
            attributes: None,
            logger: Self::get_logger(ctx),
        }
    }

    fn get_logger(ctx: Arc<Context>) -> Option<Arc<Logger>> {
        if let Some(log) = &ctx.definitions().log {
            if log.auto_log_error {
                return Some(ctx.logger().clone())
            }
        }

        None
    }

    pub fn internal(ctx: Arc<Context>, msg: &str) -> Self {
        Self::new(
            ctx,
            Error::Internal(msg.to_string()),
        )
    }

    pub fn with_code(mut self, code: i32) -> Self {
        self.code = code;
        self
    }

    pub fn with_attributes(mut self, attributes: serde_json::Value) -> Self {
        self.attributes = Some(attributes);
        self
    }

    fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap_or("could not serialize error message".to_string())
    }
}

impl From<ServiceError> for tonic::Status {
    fn from(error: ServiceError) -> Self {
        let message = error.serialize();

        // Should we log our message?
        if let Some(_logger) = error.logger {
        }

        // Return our error always as an (gRPC) Unknown?
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
            "RPCError" => StatusCode::INTERNAL_SERVER_ERROR,
            "CustomError" => StatusCode::INTERNAL_SERVER_ERROR,
            "PermissionError" => StatusCode::FORBIDDEN,
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