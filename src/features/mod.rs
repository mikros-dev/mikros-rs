use crate::features::example::Example;
use crate::plugin;

pub mod example;

/// Registers features to be used by services.
pub(crate) fn register_features() -> Vec<Box<dyn plugin::feature::Feature>>{
    vec![Box::new(Example)]
}
