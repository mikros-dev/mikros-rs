use std::sync::Arc;

use crate::errors as merrors;
use crate::service::context::Context;

/// Responsible for retrieving a value from an HTTP header map and returning
/// it as a bool.
pub fn to_bool(ctx: Arc<Context>, headers: &http::HeaderMap<http::HeaderValue>, key: &str) -> Result<bool, merrors::ServiceError> {
    if let Some(value) = headers.get(key) {
        if let Ok(value) = value.to_str() {
            return match value.to_lowercase().as_str() {
                "true" | "1" => Ok(true),
                "false" | "0" => Ok(false),
                _ => Err(merrors::ServiceError::internal(ctx, format!("invalid header value {}", value).as_str()))
            }
        }
    }

    Err(merrors::ServiceError::internal(ctx, format!("missing header {}", key).as_str()))
}

/// Responsible for retrieving a value from an HTTP header map and returning
/// it as a String.
pub fn to_string(ctx: Arc<Context>, headers: &http::HeaderMap<http::HeaderValue>, key: &str) -> Result<String, merrors::ServiceError> {
    if let Some(value) = headers.get(key) {
        if let Ok(value) = value.to_str() {
            return Ok(value.to_string());
        }
    }

    Err(merrors::ServiceError::internal(ctx, format!("missing header {}", key).as_str()))
}