use std::collections::HashMap;
use std::sync::Arc;

use crate::definition::Definitions;
use crate::env::Env;
use crate::errors as merrors;
use crate::service::context::Context;

/// Feature is a set of methods that every feature must implement to be supported
/// by the framework.
#[async_trait::async_trait]
pub trait Feature: Send + FeatureClone + std::any::Any {
    /// The feature name.
    fn name(&self) -> &str;

    /// Returns internal information about the feature to be logged when the service
    /// is being initialized.
    fn info(&self) -> HashMap<String, logger::fields::FieldValue>;

    /// Returns if the feature is currently enabled or not.
    fn is_enabled(&self) -> bool;

    /// Checks if the feature can be initialized or not.
    fn can_be_initialized(&self, definitions: Arc<Definitions>, envs: Arc<Env>) -> merrors::Result<bool>;

    /// Initializes everything the feature needs to run. Also, here is the place
    /// where, if it needs, some task should be put to execute.
    async fn initialize(&mut self, ctx: &Context) -> merrors::Result<()>;

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
