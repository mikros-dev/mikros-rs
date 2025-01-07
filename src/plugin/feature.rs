use std::sync::Arc;

use crate::definition::Definitions;
use crate::env::Env;
use crate::errors;
use crate::service::context::Context;

/// Feature is a set of methods that every feature must implement to be supported
/// by the framework.
///
/// The API described by this trait is only used internally. So if the service
/// requires accessing the feature public API (which should be returned by the
/// service_api method), one can do the following:
///
/// ```ignore
/// use mikros::service::context::Context;
/// use mikros::{errors as merrors, plugin};
///
/// pub trait ExampleAPI {
///     fn do_something(&self);
/// }
///
/// pub struct Example;
///
/// /// Retrieves the feature API to be used inside a service and, if found, calls
/// /// a closure over with.
/// pub async fn execute_on<F>(ctx: &Context, f: F) -> merrors::Result<()>
/// where
///     F: FnOnce(&dyn ExampleAPI) -> merrors::Result<()>,
/// {
///     let feature = ctx.feature("simple_api").await?;
///     if let Some(api) = to_api(&feature) {
///         f(api)?
///     }
///
///     Ok(())
/// }
///
/// fn to_api(feature: &Box<dyn plugin::feature::Feature>) -> Option<&dyn ExampleAPI> {
///     feature.service_api()?.downcast_ref::<Example>().map(|s| s as &dyn ExampleAPI)
/// }
/// ```
///
/// It is recommended, to keep a standard, that the public function which returns
/// the feature public API to be named 'execute_on'.
#[async_trait::async_trait]
pub trait Feature: Send + FeatureClone + std::any::Any {
    /// The feature name.
    fn name(&self) -> &str;

    /// Returns internal information about the feature to be logged when the service
    /// is being initialized.
    fn info(&self) -> serde_json::Value;

    /// Returns if the feature is currently enabled or not.
    fn is_enabled(&self) -> bool;

    /// Checks if the feature can be initialized or not.
    fn can_be_initialized(
        &self,
        definitions: Arc<Definitions>,
        envs: Arc<Env>,
    ) -> errors::Result<bool>;

    /// Initializes everything the feature needs to run. Also, here is the place
    /// where, if it needs, some task should be put to execute.
    async fn initialize(&mut self, ctx: Arc<Context>) -> errors::Result<()>;

    /// Release resources from the feature.
    async fn cleanup(&self);

    /// Returns the feature API that should be used by services and applications.
    fn service_api(&self) -> Option<&dyn std::any::Any>;
}

pub trait FeatureClone {
    fn clone_box(&self) -> Box<dyn Feature>;
}

impl<T> FeatureClone for T
where
    T: 'static + Feature + Clone,
{
    fn clone_box(&self) -> Box<dyn Feature> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Feature> {
    fn clone(&self) -> Box<dyn Feature> {
        self.clone_box()
    }
}

/// This macro adds APIs to the feature allowing it to be used by services
/// in an easy way.
///
/// The execute_on function receives a closure that will be executed in the
/// feature, if it is enabled.
#[macro_export]
macro_rules! impl_feature_public_api {
    ($api_trait:ident, $api_struct:ident, $feature_name:expr) => {
        pub fn new() -> Box<dyn mikros::plugin::feature::Feature> {
            Box::new($api_struct::default())
        }

        pub async fn execute_on<F>(
            ctx: Arc<Context>,
            f: F,
        ) -> mikros::errors::Result<()>
        where
            F: FnOnce(&dyn $api_trait) -> mikros::errors::Result<()>,
        {
            let feature = ctx.feature($feature_name).await?;
            if let Some(api) = to_api(&feature) {
                f(api)?;
            }

            Ok(())
        }

        fn to_api(feature: &Box<dyn plugin::feature::Feature>) -> Option<&dyn $api_trait> {
            feature
                .service_api()?
                .downcast_ref::<$api_struct>()
                .map(|s| s as &dyn $api_trait)
        }
    };
}
