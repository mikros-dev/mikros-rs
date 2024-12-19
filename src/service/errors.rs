use crate::definition;

// Module internal errors
crate::internal_errors!(
    Error {
        EmptyServiceFound => "cannot execute without a service implementation",
        FeatureNotFound(f: String) => "feature '{}' not found",
        UnsupportedServicesExecutionMode => "unsupported services execution mode",
        ServiceKindUninitialized(k: definition::ServiceKind) => "service type uninitialized: {}",
        FeatureDisabled(f: String) => "feature '{}' is disabled",
        ServiceAlreadyInitialized(k: String) => "service '{}' already initialized"
    }
);