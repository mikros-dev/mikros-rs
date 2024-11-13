use crate::errors as merrors;

#[async_trait::async_trait]
pub trait Lifecycle: Send + Sync {
    async fn on_start(&mut self) -> merrors::Result<()> {
        Ok(())
    }

    async fn on_finish(&self) -> merrors::Result<()> {
        Ok(())
    }
}
