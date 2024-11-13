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

impl mikros::service::native::NativeService for Service {
    fn start(&mut self, ctx: &Context) -> merrors::Result<()> {
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

    fn stop(&mut self, ctx: &Context) {
        ctx.logger().info("Stop native service");
    }
}

impl mikros::service::script::ScriptService for Service {
    fn run(&mut self, ctx: &Context) -> merrors::Result<()> {
        ctx.logger().info("Start script service");
        features::example::retrieve(ctx, |api| {
            api.do_something();
        })?;

        Ok(())
    }

    fn cleanup(&mut self, ctx: &Context) {
        ctx.logger().info("Stop script service");
    }
}