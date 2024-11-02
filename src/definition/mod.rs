mod validation;

use validator::{ValidateArgs};
use crate::errors::{Result, Error};
use validation::validate_service_info;

// ServiceInfo represents the service information loaded from the 'service.toml'
// file.
#[derive(serde_derive::Deserialize, validator::Validate)]
#[validate(context = CustomServiceInfo)]
#[validate(schema(function = "validate_service_info", skip_on_field_errors = false, use_context))]
pub(crate) struct ServiceDefinitions {
    pub name: String,
    pub types: Vec<String>,
    pub version: String,
    pub language: String,
    pub product: String,
    pub envs: Option<Vec<String>>,
    pub log: Option<Log>,
}

#[derive(serde_derive::Deserialize)]
pub(crate) struct Log {
    pub level: String
}

#[derive(serde_derive::Deserialize, PartialEq)]
pub(crate) enum ServiceKind {
    Grpc,
    Http,
    Script,
    Native,
}

#[derive(Default)]
pub(crate) struct CustomServiceInfo {
    pub types: Option<Vec<String>>,
}

impl ServiceDefinitions {
    pub fn new(filename: Option<&str>, custom_info: Option<CustomServiceInfo>) -> Result<Self> {
        let info: ServiceDefinitions = match toml::from_str(&Self::load_service_file(filename)?) {
            Ok(content) => content,
            Err(e) => return Err(Error::InvalidSettings(e.to_string())),
        };

        let context = custom_info.unwrap_or_else(|| {
            let c: CustomServiceInfo = Default::default();
            c
        });

        // Then validate the final service definitions
        if let Err(e) = info.validate_with_args(&context) {
            return Err(Error::InvalidSettings(e.to_string()));
        }

        Ok(info)
    }

    fn load_service_file(filename: Option<&str>) -> Result<String> {
        let path = Self::get_service_file_path(filename)?;

        match std::fs::read_to_string(path.as_path()) {
            Ok(content) => Ok(content),
            Err(e) => Err(Error::InvalidSettings(e.to_string())),
        }
    }

    fn get_service_file_path(filename: Option<&str>) -> Result<std::path::PathBuf> {
        if let Some(filename) = filename {
            return Ok(std::path::Path::new(filename).to_path_buf())
        }

        match std::env::current_dir() {
            Ok(mut p) => {
                p.push("service.toml");
                Ok(p)
            }
            Err(r) => Err(Error::InvalidSettings(r.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assets_path() -> std::path::PathBuf {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("resources/test");
        p
    }

    #[test]
    fn test_load_service_file_with_invalid_settings() {
        let filename = assets_path().join("definitions/service.toml.err_unsupported_type");
        let defs = ServiceDefinitions::new(filename.to_str(), None);
        assert!(defs.is_err());
    }

    #[test]
    fn test_load_service_file_with_invalid_toml() {
        let filename = assets_path().join("definitions/service.toml.err_invalid_toml");
        let defs = ServiceDefinitions::new(filename.to_str(), None);
        assert!(defs.is_err());
    }

    #[test]
    fn test_load_service_file_ok() {
        let filename = assets_path().join("definitions/service.toml.ok");
        let defs = ServiceDefinitions::new(filename.to_str(), None);
        assert!(defs.is_ok());
        assert_eq!(defs.unwrap().types.len(), 1);
    }

    #[test]
    fn test_load_service_file_ok_hybrid() {
        let filename = assets_path().join("definitions/service.toml.ok_hybrid");
        let defs = ServiceDefinitions::new(filename.to_str(), None);
        assert!(defs.is_ok());

        let info = defs.unwrap();
        assert_eq!(info.types.len(), 2);
        assert_eq!(info.envs.unwrap().len(), 2);
    }

    #[test]
    fn test_load_service_file_with_custom_supported_type() {
        let custom_info = CustomServiceInfo{
            types: Some(vec!["websocket".to_string()]),
        };

        let filename = assets_path().join("definitions/service.toml.err_unsupported_type");
        let defs = ServiceDefinitions::new(filename.to_str(), Some(custom_info));
        assert!(defs.is_ok());
        assert_eq!(defs.unwrap().types.len(), 1);
    }
}
