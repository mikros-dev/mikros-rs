use mikros::errors as merrors;
use mikros::service::context::Context;
use mikros::features;

#[derive(Clone)]
pub struct Service;

impl Service {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl mikros::service::lifecycle::Lifecycle for Service {
    async fn on_start(&mut self) -> merrors::Result<()> {
        println!("lifecycle on_start");
        Ok(())
    }

    async fn on_finish(&self) -> merrors::Result<()> {
        println!("lifecycle on_finish");
        Ok(())
    }
}

impl mikros::service::native::NativeService for Service {
    fn start(&self, ctx: &Context) -> merrors::Result<()> {
        ctx.logger().info("Start native service");
//        Err(merrors::Error::InternalServiceError("some internal error happened".to_string()))

        let value = ctx.env("CUSTOM_ENV").unwrap();
        ctx.logger().info(format!("env value '{}'", value).as_str());

        features::example::retrieve(ctx, |api| {
            api.do_something();
        })?;

        ctx.logger().info("finished start native service method");
        Ok(())
    }

    fn stop(&self, ctx: &Context) {
        ctx.logger().info("Stop native service");
    }
}

impl mikros::service::script::ScriptService for Service {
    fn run(&self, ctx: &Context) -> merrors::Result<()> {
        ctx.logger().info("Start script service");
        features::example::retrieve(ctx, |api| {
            api.do_something();
        })?;

        Ok(())
    }

    fn cleanup(&self, ctx: &Context) {
        ctx.logger().info("Stop script service");
    }
}