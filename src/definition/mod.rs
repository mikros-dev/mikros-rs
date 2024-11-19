pub mod service;
mod validation;

use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::str::FromStr;

use serde::de::DeserializeOwned;
use validator::{ValidateArgs};

use crate::errors as merrors;

// ServiceInfo represents the service information loaded from the 'service.toml'
// file.
#[derive(serde_derive::Deserialize, validator::Validate)]
#[validate(context = CustomServiceInfo)]
#[validate(schema(function = "validation::validate_service_info", skip_on_field_errors = false, use_context))]
pub struct Definitions {
    pub name: String,
    pub version: String,
    pub language: String,
    pub product: String,
    pub envs: Option<Vec<String>>,
    pub log: Option<Log>,

    features: Option<HashMap<String, serde_json::Value>>,

    #[serde(deserialize_with = "service::deserialize_services")]
    pub types: Vec<service::Service>,
}

#[derive(serde_derive::Deserialize)]
pub struct Log {
    pub level: String
}

#[derive(serde_derive::Deserialize, Clone, Debug, PartialEq)]
pub enum ServiceKind {
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

impl Display for ServiceKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceKind::Grpc => write!(f, "grpc"),
            ServiceKind::Http => write!(f, "http"),
            ServiceKind::Native => write!(f, "native"),
            ServiceKind::Script => write!(f, "script"),
            ServiceKind::Custom(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Default)]
pub struct CustomServiceInfo {
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

    pub(crate) fn is_script_service(&self) -> bool {
        self.types.iter().any(|t| {
            let service::Service(kind, _) = t;
            kind.to_string() == "script"
        })
    }

    pub(crate) fn get_service_type(&self, kind: ServiceKind) -> merrors::Result<&service::Service> {
        match self.types.iter().find(|t| t.0 == kind) {
            Some(t) => Ok(t),
            None => Err(merrors::Error::NotFound(format!("could not find service kind '{}'", kind)))
        }
    }

    /// Loads definitions from a feature.
    pub fn load_feature<T>(&self, feature: &str) -> merrors::Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        if let Some(d) = self.feature(feature) {
            return match serde_json::from_value::<T>(d.clone()) {
                Err(e) => Err(merrors::Error::LoadFeatureDefinition(feature.to_string(), e.to_string())),
                Ok(defs) => Ok(Some(defs)),
            }
        }

        Ok(None)
    }

    fn feature(&self, feature: &str) -> Option<serde_json::Value> {
        match &self.features {
            None => None,
            Some(features) => features.get(feature).cloned()
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
