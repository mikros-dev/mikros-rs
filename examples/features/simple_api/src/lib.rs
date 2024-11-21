use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use mikros::env::Env;
use mikros::{errors as merrors, impl_feature_public_api};
use mikros::logger;
use mikros::plugin;
use mikros::service::context::Context;
use serde::Deserialize;

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

    fn is_enabled(&self) -> bool {
        self.definitions.enabled
    }
    
    fn can_be_initialized(&self, definitions: Arc<mikros::definition::Definitions>, _: Arc<Env>) -> merrors::Result<bool> {
        println!("simple_api can_be_initialized");

        if let Some(defs) = definitions.load_feature::<Definitions>(self.name())? {
            return Ok(defs.enabled);
        }

        Ok(false)
    }

    async fn initialize(&mut self, ctx: &Context) -> merrors::Result<()> {
        println!("simple_api initialize");
        ctx.logger().info("simple_api initialize");

        if let Some(defs) = ctx.definitions().load_feature::<Definitions>(self.name())? {
            self.definitions = defs;
        }

        Ok(())
    }

    fn service_api(&self) -> Option<&dyn Any> {
        Some(self)
    }

    fn info(&self) -> HashMap<String, logger::fields::FieldValue> {
        let collections = self.definitions.collections.join(",");
        logger::fields![
            "test" => logger::fields::FieldValue::String("Hello world".to_string()),
            "collections" => logger::fields::FieldValue::String(collections),
        ]
    }

    async fn cleanup(&self) {
        println!("simple_api cleanup");
    }
}

impl ExampleAPI for Example {
    fn do_something(&self) {
        println!("simple_api doing something...")
    }
}

impl_feature_public_api!(ExampleAPI, Example, "simple_api");
