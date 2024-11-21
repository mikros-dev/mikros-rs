use mikros::errors as merrors;
use mikros::features;
use mikros::service::context::Context;

#[derive(Clone)]
pub struct Service;

impl Service {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl mikros::service::lifecycle::Lifecycle for Service {
    async fn on_start(&mut self, _ctx: &Context) -> merrors::Result<()> {
        println!("lifecycle on_start");
        Ok(())
    }

    async fn on_finish(&self) -> merrors::Result<()> {
        println!("lifecycle on_finish");
        Ok(())
    }
}

#[async_trait::async_trait]
impl mikros::service::script::ScriptService for Service {
    async fn run(&self, ctx: &Context) -> merrors::Result<()> {
        ctx.logger().info("Start script service");
        features::example::execute_on(ctx, |api| {
            api.do_something();
            Ok(())
        })
        .await?;

        Ok(())
    }

    async fn cleanup(&self, ctx: &Context) {
        ctx.logger().info("Stop script service");
    }
}
