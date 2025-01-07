use crate::definition::{service, CustomServiceInfo, Definitions};

pub(crate) fn validate_service_info(
    info: &Definitions,
    context: &CustomServiceInfo,
) -> Result<(), validator::ValidationError> {
    validate_service_types(&info.types, context)?;
    Ok(())
}

fn validate_service_types(
    types: &Vec<service::Service>,
    context: &CustomServiceInfo,
) -> Result<(), validator::ValidationError> {
    if types.is_empty() {
        return Err(validator::ValidationError::new("no service types defined"));
    }

    for t in types {
        validate_service_type(t, context)?
    }

    Ok(())
}

fn validate_service_type(
    value: &service::Service,
    context: &CustomServiceInfo,
) -> Result<(), validator::ValidationError> {
    let service::Service(kind, _) = value;
    let mut supported_services = vec!["grpc", "http", "native", "script"];
    if let Some(types) = context.types.as_ref() {
        for t in types {
            supported_services.push(t);
        }
    }

    if supported_services.iter().any(|&e| e == kind.to_string()) {
        return Ok(());
    }

    Err(validator::ValidationError::new(
        "service type is not supported",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::ServiceKind;

    #[test]
    fn test_validate_service_type() {
        assert!(validate_service_type(
            &service::Service(ServiceKind::Custom("".to_string()), None),
            &CustomServiceInfo::default()
        )
        .is_err());

        assert!(validate_service_type(
            &service::Service(ServiceKind::Custom("cronjob".to_string()), None),
            &CustomServiceInfo::default()
        )
        .is_err());

        assert!(validate_service_type(
            &service::Service(ServiceKind::Custom("grpc".to_string()), None),
            &CustomServiceInfo::default()
        )
        .is_ok());
    }

    #[test]
    fn test_validate_service_types() {
        let http = service::Service(ServiceKind::Http, None);
        let grpc = service::Service(ServiceKind::Grpc, None);
        let grpc2 = service::Service(ServiceKind::Custom("grpc2".to_string()), None);

        assert!(validate_service_types(&Vec::new(), &CustomServiceInfo::default()).is_err());
        assert!(
            validate_service_types(&vec![http.clone(), grpc2], &CustomServiceInfo::default())
                .is_err()
        );

        assert!(validate_service_types(&vec![http, grpc], &CustomServiceInfo::default()).is_ok());
    }
}
