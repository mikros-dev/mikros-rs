
pub trait Feature: FeatureClone + std::any::Any {
    /// The feature name.
    fn name(&self) -> &str;

    /// Tells if the feature is currently enabled ot not.
    fn is_enabled(&self) -> bool;

    /// Checks if the feature can be initialized or not.
    fn can_be_initialized(&self) -> bool;

    /// Initializes the feature.
    fn init(&mut self);

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
