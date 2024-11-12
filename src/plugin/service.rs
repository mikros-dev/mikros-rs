use std::collections::HashMap;

use crate::{definition, errors as merrors};
use crate::service::context::Context;

#[async_trait::async_trait]
pub trait Service: Send + ServiceClone {
    fn kind(&self) -> definition::ServiceKind;
    fn initialize(&mut self) -> merrors::Result<()>;
    fn information(&self) -> HashMap<String, logger::fields::FieldValue>;

    async fn run(&mut self, ctx: &Context) -> merrors::Result<()>;
    async fn stop(&mut self, ctx: &Context);
}

pub trait ServiceClone {
    fn clone_box(&self) -> Box<dyn Service>;
}

impl<T> ServiceClone for T
where
    T: 'static + Service + Clone,
{
    fn clone_box(&self) -> Box<dyn Service> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Service> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
