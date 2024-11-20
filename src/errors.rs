use crate::definition;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    InvalidDefinitions(String),
    EnvironmentVariableFailure(String),
    UnsupportedServiceType,
    EmptyServiceFound,
    FeatureNotFound(String),
    InternalServiceError(String),
    UnsupportedServicesCombination,
    ServiceKindUninitialized(definition::ServiceKind),
    NotFound(String),
    DefinitionLoadingFailure(String, String),
    FeatureDisabled(String),
    BuilderFailed(String),
}

impl Error {
    fn description(&self) -> String {
        match self {
            Error::InvalidDefinitions(s) => format!("invalid service definitions: {}", s),
            Error::EnvironmentVariableFailure(s) => format!("environment variable failure: {}", s),
            Error::UnsupportedServiceType => "unsupported service type".to_string(),
            Error::EmptyServiceFound => "cannot execute without a service implementation".to_string(),
            Error::FeatureNotFound(s) => format!("feature {} not found", s),
            Error::InternalServiceError(s) => format!("internal service error: {}", s),
            Error::UnsupportedServicesCombination => "unsupported services combination".to_string(),
            Error::ServiceKindUninitialized(s) => format!("service type uninitialized: {}", s),
            Error::NotFound(s) => format!("not found: {}", s),
            Error::DefinitionLoadingFailure(s, msg) => format!("definition loading from '{}' failed: {}", s, msg),
            Error::FeatureDisabled(s) => format!("feature disabled: {}", s),
            Error::BuilderFailed(s) => format!("builder failed: {}", s),
        }
    }
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}