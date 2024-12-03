use std::any::Any;
use std::sync::Arc;

use mikros::definition::Definitions;
use mikros::env::Env;
use mikros::{errors as merrors, impl_feature_public_api, plugin, serde_json};
use mikros::service::context::Context;

/// The feature public API.
pub trait ExampleAPI {
    fn do_something(&self);
}

#[derive(Clone, Default)]
pub(crate) struct Example;

#[async_trait::async_trait]
impl plugin::feature::Feature for Example {
    fn name(&self) -> &str {
        "example"
    }

    fn info(&self) -> serde_json::Value {
        serde_json::json!({
            "test": "Hello world".to_string(),
        })
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn can_be_initialized(&self, _definitions: Arc<Definitions>, _envs: Arc<Env>) -> merrors::Result<bool> {
        println!("example can_be_initialized");
        Ok(true)
    }

    async fn initialize(&mut self, _ctx: &Context) -> merrors::Result<()> {
        println!("example initialized");
        Ok(())
    }

    async fn cleanup(&self) {
        println!("example cleanup");
    }

    fn service_api(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

impl ExampleAPI for Example {
    fn do_something(&self) {
        println!("something")
    }
}

// Use this macro to export a public API equal for all features, allowing the
// services to use the same syntax while accessing them.
impl_feature_public_api!(ExampleAPI, Example, "example");
