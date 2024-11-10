use mikros::errors as merrors;
use mikros::service::context::Context;
use mikros::features;

#[derive(Clone, Default)]
pub struct Service {
    value: String,
}

impl mikros::service::native::NativeService for Service {
    fn start(&mut self, ctx: &Context) -> merrors::Result<()> {
        ctx.logger().info("Start native service");

        let value = ctx.env("CUSTOM_ENV").unwrap();
        ctx.logger().info(format!("env value '{}'", value).as_str());

        if let Some(api) = features::example::retrieve(ctx)? {
            api.do_something();
        }

        Ok(())
    }

    fn stop(&mut self, ctx: &Context) {
        ctx.logger().info("Stop native service");
    }
}

impl mikros::service::script::ScriptService for Service {
    fn run(&mut self, ctx: &Context) -> merrors::Result<()> {
        ctx.logger().info("Start script service");
        if let Some(api) = features::example::retrieve(ctx)? {
            api.do_something();
        }

        Ok(())
    }

    fn cleanup(&mut self, ctx: &Context) {
        ctx.logger().info("Stop script service");
    }
}