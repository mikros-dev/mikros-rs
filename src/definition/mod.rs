mod validation;

use std::sync::Arc;
use std::str::FromStr;
use validator::{ValidateArgs};

use crate::errors as merrors;

// ServiceInfo represents the service information loaded from the 'service.toml'
// file.
#[derive(serde_derive::Deserialize, validator::Validate)]
#[validate(context = CustomServiceInfo)]
#[validate(schema(function = "validation::validate_service_info", skip_on_field_errors = false, use_context))]
pub(crate) struct Definitions {
    pub name: String,
    pub version: String,
    pub language: String,
    pub product: String,
    pub envs: Option<Vec<String>>,
    pub log: Option<Log>,
//    pub types2: Vec<Service>,
    types: Vec<String>,
}

#[derive(serde_derive::Deserialize)]
pub(crate) struct Log {
    pub level: String
}

pub(crate) struct Service(ServiceKind, i32);

#[derive(serde_derive::Deserialize)]
pub(crate) enum ServiceKind {
    Grpc,
    Http,
    Script,
    Native,
    Custom(String),
}

impl FromStr for ServiceKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "grpc" => Ok(ServiceKind::Grpc),
            "http" => Ok(ServiceKind::Http),
            "native" => Ok(ServiceKind::Native),
            "script" => Ok(ServiceKind::Script),
            _ => Ok(ServiceKind::Custom(s.to_string())),
        }
    }
}

#[derive(Default)]
pub(crate) struct CustomServiceInfo {
    pub types: Option<Vec<String>>,
}

impl Definitions {
    pub fn new(filename: Option<&str>, custom_info: Option<CustomServiceInfo>) -> merrors::Result<Arc<Self>> {
        let info: Definitions = match toml::from_str(&Self::load_service_file(filename)?) {
            Ok(content) => content,
            Err(e) => return Err(merrors::Error::InvalidDefinitions(e.to_string())),
        };

        let context = custom_info.unwrap_or_else(|| {
            let c: CustomServiceInfo = Default::default();
            c
        });

        if let Err(e) = info.validate_with_args(&context) {
            return Err(merrors::Error::InvalidDefinitions(e.to_string()));
        }

        Ok(Arc::new(info))
    }

    fn load_service_file(filename: Option<&str>) -> merrors::Result<String> {
        let path = Self::get_service_file_path(filename)?;

        match std::fs::read_to_string(path.as_path()) {
            Ok(content) => Ok(content),
            Err(e) => Err(merrors::Error::InvalidDefinitions(e.to_string())),
        }
    }

    fn get_service_file_path(filename: Option<&str>) -> merrors::Result<std::path::PathBuf> {
        if let Some(filename) = filename {
            return Ok(std::path::Path::new(filename).to_path_buf())
        }

        match std::env::current_dir() {
            Ok(mut p) => {
                p.push("service.toml");
                Ok(p)
            }
            Err(r) => Err(merrors::Error::InvalidDefinitions(r.to_string())),
        }
    }

    pub(crate) fn is_service_configured(&self, service_type: &str) -> bool {
        self.types.iter().any(|t| t == service_type)
    }

    pub fn service_types(&self) -> Vec<ServiceKind> {
        let mut types: Vec<ServiceKind> = Vec::new();
        for t in self.types.iter() {
            types.push(ServiceKind::from_str(t).unwrap());
        }

        types
    }

    pub(crate) fn is_script_service(&self) -> bool {
        self.types.iter().any(|t| t.as_str() == "script")
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
        let defs = Definitions::new(filename.to_str(), None);
        assert!(defs.is_err());
    }

    #[test]
    fn test_load_service_file_with_invalid_toml() {
        let filename = assets_path().join("definitions/service.toml.err_invalid_toml");
        let defs = Definitions::new(filename.to_str(), None);
        assert!(defs.is_err());
    }

    #[test]
    fn test_load_service_file_ok() {
        let filename = assets_path().join("definitions/service.toml.ok");
        let defs = Definitions::new(filename.to_str(), None);
        assert!(defs.is_ok());
        assert_eq!(defs.unwrap().types.len(), 1);
    }

    // FIXME
    #[test]
    fn test_load_service_file_ok_hybrid() {
        let filename = assets_path().join("definitions/service.toml.ok_hybrid");
        let defs = Definitions::new(filename.to_str(), None);
        assert!(defs.is_ok());

        match defs {
            Ok(info ) => {
                assert_eq!(info.types.len(), 2);
                assert_eq!(info.envs.clone().unwrap().len(), 2);
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_load_service_file_with_custom_supported_type() {
        let custom_info = CustomServiceInfo{
            types: Some(vec!["websocket".to_string()]),
        };

        let filename = assets_path().join("definitions/service.toml.err_unsupported_type");
        let defs = Definitions::new(filename.to_str(), Some(custom_info));
        assert!(defs.is_ok());
        assert_eq!(defs.unwrap().types.len(), 1);

    }
}
