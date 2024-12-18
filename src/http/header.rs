use crate::errors as merrors;

pub fn to_bool(headers: &http::HeaderMap<http::HeaderValue>, key: &str) -> merrors::Result<bool> {
    if let Some(value) = headers.get(key) {
        if let Ok(value) = value.to_str() {
            return match value.to_lowercase().as_str() {
                "true" | "1" => Ok(true),
                "false" | "0" => Ok(false),
                _ => Err(merrors::Error::InvalidHeaderValue(key.to_string())),
            }
        }
    }

    Err(merrors::Error::HeaderMissing(key.to_string()))
}

pub fn to_str(headers: &http::HeaderMap<http::HeaderValue>, key: &str) -> merrors::Result<String> {
    if let Some(value) = headers.get(key) {
        if let Ok(value) = value.to_str() {
            return Ok(value.to_string());
        }
    }

    Err(merrors::Error::HeaderMissing(key.to_string()))
}