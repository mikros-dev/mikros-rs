
use crate::definition::{ServiceDefinitions, CustomServiceInfo};

pub(crate) fn validate_service_info(info: &ServiceDefinitions, context: &CustomServiceInfo) -> Result<(), validator::ValidationError> {
    validate_service_types(&info.types, context)?;
    Ok(())
}

fn validate_service_types(types: &Vec<String>, context: &CustomServiceInfo) -> Result<(), validator::ValidationError> {
    if types.is_empty() {
        return Err(validator::ValidationError::new("no service types defined"));
    }

    for t in types {
        validate_service_type(t, context)?
    }

    Ok(())
}

fn validate_service_type(value: &str, context: &CustomServiceInfo) -> Result<(), validator::ValidationError> {
    let mut supported_services = vec!["grpc", "http", "native", "script"];
    if let Some(types) = context.types.as_ref() {
        for t in types {
            supported_services.push(t);
        }
    }

    if supported_services.iter().any(|&e| e == value) {
        return Ok(());
    }

    Err(validator::ValidationError::new("service type is not supported"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_service_type() {
        assert!(validate_service_type("", &CustomServiceInfo::default()).is_err());
        assert!(validate_service_type("cronjob", &CustomServiceInfo::default()).is_err());
        assert!(validate_service_type("grpc", &CustomServiceInfo::default()).is_ok());
    }

    #[test]
    fn test_validate_service_types() {
        assert!(validate_service_types(&Vec::new(), &CustomServiceInfo::default()).is_err());
        assert!(validate_service_types(&vec!["http".to_string(), "grpc2".to_string()], &CustomServiceInfo::default()).is_err());
        assert!(validate_service_types(&vec!["http".to_string(), "grpc".to_string()], &CustomServiceInfo::default()).is_ok());
    }
}