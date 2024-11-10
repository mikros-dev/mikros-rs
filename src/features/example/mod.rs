use std::any::Any;

use crate::errors as merrors;
use crate::plugin;
use crate::service::context::Context;

/// The feature public API.
pub trait ExampleAPI {
    fn do_something(&self);
}

/// Retrieves the feature API to be used inside a service.
pub fn retrieve(ctx: &Context) -> merrors::Result<Option<&dyn ExampleAPI>> {
    let name = "example";

    match ctx.feature(name) {
        Some(f) => Ok(retrieve_example_api(f)),
        None => Err(merrors::Error::FeatureNotFound(name.to_string())),
    }
}

fn retrieve_example_api(feature: &Box<dyn plugin::feature::Feature>) -> Option<&dyn ExampleAPI> {
    feature.service_api()?.downcast_ref::<Example>().map(|s| s as &dyn ExampleAPI)
}

#[derive(Clone, Default)]
pub(crate) struct Example;

impl plugin::feature::Feature for Example {
    fn name(&self) -> &str {
        "example"
    }

    fn is_enabled(&self) -> bool {
        todo!()
    }

    fn can_be_initialized(&self) -> bool {
        todo!()
    }

    fn init(&mut self) {
        todo!()
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
