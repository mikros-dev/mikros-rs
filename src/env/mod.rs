mod errors;

use std::sync::Arc;
use env_settings_derive::EnvSettings;

use crate::definition::Definitions;

#[derive(EnvSettings, Debug)]
#[env_settings(delay)]
pub struct Env {
    #[env_settings(variable = "MIKROS_SERVICE_DEPLOY", default = "local")]
    pub deployment_env: String,

    #[env_settings(variable = "MIKROS_TRACKER_HEADER_NAME", default = "X-Request-ID")]
    pub tracker_header_name: String,

    #[env_settings(variable = "MIKROS_COUPLED_NAMESPACE", default = "localhost")]
    pub coupled_namespace: String,

    #[env_settings(variable = "MIKROS_COUPLED_PORT", default = "7070")]
    pub coupled_port: String,

    #[env_settings(variable = "MIKROS_GRPC_PORT", default = "7070")]
    pub grpc_port: i32,

    #[env_settings(variable = "MIKROS_HTTP_PORT", default = "8080")]
    pub http_port: i32,

    #[env_settings(variable = "MIKROS_HIDE_RESPONSE_FIELDS", default = "")]
    pub hide_response_fields: Option<String>,

    #[env_settings(skip)]
    defined_envs: std::collections::HashMap<String, String>,
}

impl Env {
    pub fn load(defs: &Definitions) -> Result<Arc<Self>, errors::Error> {
        match Env::from_env(std::collections::HashMap::new()) {
            Err(e) => Err(errors::Error::SettingsError(e.to_string())),
            Ok(mut env) => {
                env.defined_envs = env.load_defined_envs(defs)?;
                Ok(Arc::new(env))
            },
        }
    }

    fn load_defined_envs(&self, defs: &Definitions) -> Result<std::collections::HashMap<String, String>, errors::Error> {
        let mut envs = std::collections::HashMap::new();

        if let Some(defined_envs) = &defs.envs {
            for e in defined_envs {
                envs.insert(e.clone(), match std::env::var(e) {
                    Ok(v) => v,
                    Err(_) => return Err(errors::Error::VariableNotSet(e.to_string())),
                });
            }
        }

        Ok(envs)
    }
    
    pub fn get_defined_env(&self, name: &str) -> Option<&String> {
        self.defined_envs.get(name)
    }

    pub fn response_fields(&self) -> Option<Vec<String>> {
        self.hide_response_fields
            .as_ref()
            .map(|fields| fields.split(',').map(String::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::Definitions;

    fn assets_path() -> std::path::PathBuf {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("resources/test");
        p
    }

    #[test]
    fn test_load_env() {
        std::env::set_var("MIKROS_COUPLED_NAMESPACE", "127.0.0.1".to_string());
        let filename = assets_path().join("definitions/service.toml.ok");
        let defs = Definitions::new(filename.to_str(), None).unwrap();
        let e = Env::load(&defs).unwrap();
        assert_eq!(e.coupled_namespace, "127.0.0.1".to_string());
    }
}