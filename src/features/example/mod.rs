use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use logger::fields::FieldValue;

use crate::definition::Definitions;
use crate::env::Env;
use crate::errors as merrors;
use crate::plugin;
use crate::service::context::Context;

/// The feature public API.
pub trait ExampleAPI {
    fn do_something(&self);
}

/// Retrieves the feature API to be used inside a service and, if found, calls
/// a closure over with.
pub async fn execute_on<F>(ctx: &Context, f: F) -> merrors::Result<()>
where
    F: FnOnce(&dyn ExampleAPI) -> merrors::Result<()>,
{
    let feature = ctx.feature("simple_api").await?;
    if let Some(api) = to_api(&feature) {
        f(api)?
    }

    Ok(())
}

fn to_api(feature: &Box<dyn plugin::feature::Feature>) -> Option<&dyn ExampleAPI> {
    feature.service_api()?.downcast_ref::<Example>().map(|s| s as &dyn ExampleAPI)
}

#[derive(Clone, Default)]
pub(crate) struct Example;

#[async_trait::async_trait]
impl plugin::feature::Feature for Example {
    fn name(&self) -> &str {
        "example"
    }

    fn info(&self) -> HashMap<String, FieldValue> {
        logger::fields![
            "test" => FieldValue::String("Hello world".to_string()),
        ]
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
