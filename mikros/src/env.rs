mod errors;

use std::sync::Arc;
use std::collections::HashMap;
use mikros_macros::Env;

use crate::definition::Definitions;

#[derive(Env, Debug)]
pub struct Env {
    #[env(variable = "MIKROS_SERVICE_DEPLOY", default = "local")]
    pub deployment_env: String,

    #[env(variable = "MIKROS_TRACKER_HEADER_NAME", default = "X-Request-ID")]
    pub tracker_header_name: String,

    #[env(variable = "MIKROS_COUPLED_NAMESPACE", default = "localhost")]
    pub coupled_namespace: String,

    #[env(variable = "MIKROS_COUPLED_PORT", default = "7070")]
    pub coupled_port: String,

    #[env(variable = "MIKROS_GRPC_PORT", default = "7070")]
    pub grpc_port: i32,

    #[env(variable = "MIKROS_HTTP_PORT", default = "8080")]
    pub http_port: i32,

    #[env(variable = "MIKROS_HIDE_RESPONSE_FIELDS", default = "")]
    pub hide_response_fields: Option<String>,

    #[env(skip)]
    defined_envs: HashMap<String, String>,
}

impl Env {
    pub fn load(defs: &Definitions) -> Result<Arc<Self>, errors::Error> {
        let mut env = Env::from_env();
        env.defined_envs = Self::load_defined_envs(defs)?;

        // Load the same values but using the service as suffix to override the
        // previous values.
        let svc_env = Env::from_env_with_suffix(&defs.name);

        Ok(Arc::new(env.merge(svc_env)))
    }

    fn load_defined_envs(
        defs: &Definitions,
    ) -> Result<HashMap<String, String>, errors::Error> {
        let mut envs = HashMap::new();

        if let Some(defined_envs) = &defs.envs {
            for e in defined_envs {
                envs.insert(
                    e.clone(),
                    match std::env::var(e) {
                        Ok(v) => v,
                        Err(_) => return Err(errors::Error::VariableNotSet(e.to_string())),
                    },
                );
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

    fn merge(self, other: Env) -> Self {
        Self{
            deployment_env: Self::string_other(&other.deployment_env, &self.deployment_env),
            tracker_header_name: Self::string_other(&other.tracker_header_name, &self.tracker_header_name),
            coupled_namespace: Self::string_other(&other.coupled_namespace, &self.coupled_namespace),
            coupled_port: Self::string_other(&other.coupled_port, &self.coupled_port),
            grpc_port: other.grpc_port | self.grpc_port,
            http_port: other.http_port | self.http_port,
            hide_response_fields: other.hide_response_fields.or(self.hide_response_fields),
            defined_envs: self.defined_envs,
        }
    }

    fn string_other(a: &str, b: &str) -> String {
        println!("a:{a}, b:{b}");
        if !a.is_empty() {
            return a.to_string()
        }

        b.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::Definitions;

    fn assets_path() -> std::path::PathBuf {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.pop();
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
