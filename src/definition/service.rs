use std::fmt::Formatter;

use serde::de::{Deserializer, Visitor, Error as SerdeError};
use serde::{Deserialize, de::IntoDeserializer};

use crate::definition::ServiceKind;

#[derive(Debug, Clone)]
pub struct Service(pub ServiceKind, pub Option<i32>);

impl<'a> Deserialize<'a> for Service {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>
    {
        struct ServiceVisitor;

        impl Visitor<'_> for ServiceVisitor {
            type Value = Service;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a service type in the format `name` or `name:port`")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: SerdeError
            {
                let parts: Vec<&str> = v.split(':').collect();
                let service_kind = match parts[0] {
                    "grpc" => ServiceKind::Grpc,
                    "http" => ServiceKind::Http,
                    "native" => ServiceKind::Native,
                    "script" => ServiceKind::Script,
                    _ => ServiceKind::Custom(parts[0].to_string())
                };

                let port = if parts.len() > 1 {
                    Some(parts[1].parse::<i32>().map_err(|_| E::custom("invalid port number"))?)
                } else {
                    None
                };

                Ok(Service(service_kind, port))
            }
        }

        deserializer.deserialize_str(ServiceVisitor)
    }
}

// Custom deserialization function for Vec<Service>
pub(crate) fn deserialize_services<'de, D>(deserializer: D) -> Result<Vec<Service>, D::Error>
where
    D: Deserializer<'de>,
{
    let services_as_strings: Vec<String> = Vec::deserialize(deserializer)?; // Fully qualify Vec::deserialize
    services_as_strings
        .into_iter()
        .map(|s| Service::deserialize(s.into_deserializer()))
        .collect()
}


#[cfg(test)]
mod tests {
    use super::*;

    // Wrapper struct to deserialize from TOML
    #[derive(Debug, serde_derive::Deserialize)]
    struct Config {
        #[serde(deserialize_with = "deserialize_services")]
        types: Vec<Service>,
    }

    #[test]
    fn test_service_type_with_port() {
        let data = "types = [\"grpc:9090\", \"http:8080\", \"native\", \"subscriber\", \"consumer:7070\"]";
        let config: Config = toml::from_str(data).unwrap();
        assert_eq!(config.types.len(), 5);
    }

    #[test]
    fn test_service_type_without_port() {}

    #[test]
    fn test_custom_service_type_with_port() {}

    #[test]
    fn test_custom_service_type_without_port() {}
}