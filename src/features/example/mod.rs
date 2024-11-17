use std::any::Any;

use crate::errors as merrors;
use crate::plugin;
use crate::service::context::Context;

/// The feature public API.
pub trait ExampleAPI {
    fn do_something(&self);
}

/// Retrieves the feature API to be used inside a service and, if found, calls
/// a closure over with.
pub fn retrieve(ctx: &Context, f: impl FnOnce(&dyn ExampleAPI)) -> merrors::Result<()> {
    let name = "example";

    match ctx.features.lock().unwrap().iter().find(|f| f.name() == name) {
        None => Err(merrors::Error::FeatureNotFound(name.to_string())),
        Some(feature) => {
            if let Some(api) = retrieve_example_api(feature) {
                f(api);
            }

            Ok(())
        }
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

    fn initialize(&mut self) {
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
