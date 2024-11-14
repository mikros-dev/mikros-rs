use crate::errors as merrors;

#[async_trait::async_trait]
pub trait Lifecycle: LifecycleClone + Send + Sync {
    async fn on_start(&mut self) -> merrors::Result<()> {
        Ok(())
    }

    async fn on_finish(&self) -> merrors::Result<()> {
        Ok(())
    }
}

pub trait LifecycleClone {
    fn clone_box(&self) -> Box<dyn Lifecycle>;
}

impl<T> LifecycleClone for T
where
    T: 'static + Lifecycle + Clone,
{
    fn clone_box(&self) -> Box<dyn Lifecycle> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Lifecycle> {
    fn clone(&self) -> Self {
        LifecycleClone::clone_box(self.as_ref())
    }
}
