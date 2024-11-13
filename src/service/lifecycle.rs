use crate::errors as merrors;

#[async_trait::async_trait]
pub trait Lifecycle {
    async fn on_start(&self) -> merrors::Result<()> {
        println!("default on_start");
        Ok(())
    }

    async fn on_finish(&self) -> merrors::Result<()> {
        println!("default on_finish");
        Ok(())
    }
}
