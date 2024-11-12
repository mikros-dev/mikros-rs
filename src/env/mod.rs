use std::sync::Arc;
use env_settings_derive::EnvSettings;

use crate::definition::Definitions;
use crate::errors as merrors;

#[derive(EnvSettings)]
#[env_settings(delay)]
pub(crate) struct Env {
    #[env_settings(variable = "MIKROS_SERVICE_DEPLOY", default = "local")]
    pub deployment_env: String,

    #[env_settings(variable = "MIKROS_TRACKER_HEADER_NAME", default = "X-Request-ID")]
    pub tracker_header_name: String,

    #[env_settings(variable = "MIKROS_COUPLED_NAMESPACE")]
    pub coupled_namespace: String,

    #[env_settings(variable = "MIKROS_COUPLED_PORT", default = "7070")]
    pub coupled_port: String,

    #[env_settings(variable = "MIKROS_GRPC_PORT", default = "7070")]
    pub grpc_port: i32,

    #[env_settings(variable = "MIKROS_HTTP_PORT", default = "8080")]
    pub http_port: i32,

    #[env_settings(skip)]
    defined_envs: std::collections::HashMap<String, String>,
}

impl Env {
    pub fn load(defs: &Definitions) -> merrors::Result<Arc<Self>> {
        let e = Env::from_env(std::collections::HashMap::new());
        match e {
            Ok(mut env) => {
                env.defined_envs = env.load_defined_envs(defs)?;
                Ok(Arc::new(env))
            },
            Err(e) => Err(merrors::Error::EnvironmentVariableFailure(e.to_string()))
        }
    }

    fn load_defined_envs(&self, defs: &Definitions) -> merrors::Result<std::collections::HashMap<String, String>> {
        let mut envs = std::collections::HashMap::new();

        if let Some(defined_envs) = &defs.envs {
            for e in defined_envs {
                envs.insert(e.clone(), match std::env::var(e) {
                    Ok(v) => v,
                    Err(_) => return Err(merrors::Error::EnvironmentVariableFailure(format!("environment variable {} not set", e))),
                });
            }
        }

        Ok(envs)
    }
    
    pub fn get_defined_env(&self, name: &str) -> Option<&String> {
        self.defined_envs.get(name)
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
        std::env::set_var("MIKROS_COUPLED_NAMESPACE", "localhost".to_string());
        let filename = assets_path().join("definitions/service.toml.ok");
        let defs = Definitions::new(filename.to_str(), None).unwrap();
        let e = Env::load(&defs).unwrap();
        println!("{:?}", e);
    }
}