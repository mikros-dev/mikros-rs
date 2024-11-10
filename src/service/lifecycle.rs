
pub trait Lifecycle {
    fn on_start(&self) -> crate::errors::Result<()> {
        println!("default on_start");
        Ok(())
    }

    fn on_finish(&self) -> crate::errors::Result<()> {
        println!("default on_finish");
        Ok(())
    }
}
