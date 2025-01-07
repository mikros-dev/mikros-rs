use std::any::Any;
use std::sync::Arc;

use mikros::env::Env;
use mikros::{errors, impl_feature_public_api, serde_json};
use mikros::plugin;
use mikros::service::context::Context;
use serde_derive::Deserialize;

/// The feature public API.
pub trait ExampleAPI {
    fn do_something(&self);
}

#[derive(Clone, Default)]
pub(crate) struct Example {
    definitions: Definitions
}

#[derive(Clone, Default, Deserialize)]
pub struct Definitions {
    enabled: bool,
    collections: Vec<String>,
}

#[async_trait::async_trait]
impl plugin::feature::Feature for Example {
    fn name(&self) -> &str {
        "simple_api"
    }

    fn info(&self) -> serde_json::Value {
        let collections = self.definitions.collections.join(",");
        serde_json::json!({
            "test": "Hello world".to_string(),
            "collections": collections,
        })
    }

    fn is_enabled(&self) -> bool {
        self.definitions.enabled
    }

    fn can_be_initialized(&self, definitions: Arc<mikros::definition::Definitions>, _: Arc<Env>) -> errors::Result<bool> {
        println!("simple_api can_be_initialized");

        if let Some(defs) = definitions.load_feature::<Definitions>(self.name()) {
            return Ok(defs.enabled);
        }

        Ok(false)
    }

    async fn initialize(&mut self, ctx: Arc<Context>) -> errors::Result<()> {
        println!("simple_api initialize");
        ctx.logger().info("simple_api initialize");

        if let Some(defs) = ctx.definitions().load_feature::<Definitions>(self.name()) {
            self.definitions = defs;
        }

        Ok(())
    }

    async fn cleanup(&self) {
        println!("simple_api cleanup");
    }

    fn service_api(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

impl ExampleAPI for Example {
    fn do_something(&self) {
        println!("simple_api doing something...")
    }
}

// Use this macro to export a public API equal for all features, allowing the
// services to use the same syntax while accessing them.
impl_feature_public_api!(ExampleAPI, Example, "simple_api");
