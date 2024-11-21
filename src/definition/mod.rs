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
#[derive(serde_derive::Deserialize, validator::Validate, Debug)]
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
    services: Option<HashMap<String, serde_json::Value>>,
    clients: Option<HashMap<String, Client>>,

    #[serde(deserialize_with = "service::deserialize_services")]
    pub types: Vec<service::Service>,
}

#[derive(serde_derive::Deserialize, Debug)]
pub struct Log {
    pub level: String
}

#[derive(serde_derive::Deserialize, Debug, Clone)]
pub struct Client {
    host: String,
    port: i32,
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
                Err(e) => Err(merrors::Error::DefinitionLoadingFailure(feature.to_string(), e.to_string())),
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

    pub fn load_service<T>(&self, service_kind: ServiceKind) -> merrors::Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        if let Some(d) = self.service(&service_kind) {
            return match serde_json::from_value::<T>(d.clone()) {
                Err(e) => Err(merrors::Error::DefinitionLoadingFailure(service_kind.to_string(), e.to_string())),
                Ok(defs) => Ok(Some(defs)),
            }
        }

        Ok(None)
    }

    fn service(&self, service_kind: &ServiceKind) -> Option<serde_json::Value> {
        match &self.services {
            None => None,
            Some(services) => services.get(&service_kind.to_string()).cloned()
        }
    }

    pub fn client(&self, name: &str) -> Option<Client> {
        self.clients.clone()?.get(name).cloned()
    }
}

#[cfg(test)]
mod tests {
    use serde_derive::Deserialize;
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

    #[test]
    fn test_load_features_settings() {
        let filename = assets_path().join("definitions/service.toml.ok");
        let defs = Definitions::new(filename.to_str(), None);
        assert!(defs.is_ok());

        let defs = defs.unwrap();

        #[derive(Deserialize)]
        struct SimpleApi {
            enabled: bool,
            collections: Vec<String>,
        }

        let s: merrors::Result<Option<SimpleApi>> = defs.clone().load_feature("simple_api");
        assert!(s.is_ok());

        let simple_api = s.unwrap().unwrap();
        assert_eq!(simple_api.collections.len(), 2);
        assert_eq!(simple_api.enabled, true);

        #[derive(Deserialize)]
        struct AnotherApi {
            enabled: bool,
            use_tls: bool,
            host: String,
        }

        let s: merrors::Result<Option<AnotherApi>> = defs.clone().load_feature("another_api");
        assert!(s.is_ok());

        let another_api = s.unwrap().unwrap();
        assert_eq!(another_api.enabled, true);
        assert_eq!(another_api.use_tls, true);
        assert_eq!(another_api.host, "localhost");
    }

    #[test]
    fn test_load_service_settings() {
        let custom_info = CustomServiceInfo{
            types: Some(vec!["cronjob".to_string()]),
        };

        let filename = assets_path().join("definitions/service.toml.ok_cronjob");
        let defs = Definitions::new(filename.to_str(), Some(custom_info));
        assert!(defs.is_ok());

        let defs = defs.unwrap();

        #[derive(Deserialize)]
        struct Cronjob {
            frequency: String,
            scheduled_times: Vec<String>,
            days: Vec<String>,
        }

        let s: merrors::Result<Option<Cronjob>> = defs.clone().load_service(ServiceKind::Custom("cronjob".to_string()));
        assert!(s.is_ok());

        let cronjob = s.unwrap().unwrap();
        assert_eq!(cronjob.days.len(), 1);
        assert_eq!(cronjob.frequency, "weekly");
        assert_eq!(cronjob.scheduled_times.len(), 2);
    }

    #[test]
    fn test_load_clients() {
        let filename = assets_path().join("definitions/service.toml.ok_clients");
        let defs = Definitions::new(filename.to_str(), None);
        assert!(defs.is_ok());

        let defs = defs.unwrap();
        let user = defs.client("user");
        assert!(user.is_some());
        assert_eq!(user.clone().unwrap().host, "localhost");
        assert_eq!(user.unwrap().port, 7070);

        let auth = defs.client("auth");
        assert!(auth.is_none());

        let address = defs.client("address");
        assert!(address.is_some());
        assert_eq!(address.clone().unwrap().host, "127.0.0.1");
        assert_eq!(address.unwrap().port, 7071);
    }
}
