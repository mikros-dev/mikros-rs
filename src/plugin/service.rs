
use crate::{errors as merrors};
use crate::service::context::Context;

pub trait Service: ServiceClone {
    fn name(&self) -> &str;
    fn initialize(&mut self) -> merrors::Result<()>;
    fn run(&mut self, ctx: &Context) -> merrors::Result<()>;
    fn stop(&mut self, ctx: &Context);
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
