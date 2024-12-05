use mikros::errors as merrors;
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
impl mikros::service::native::NativeService for Service {
    async fn start(&self, ctx: &Context) -> merrors::Result<()> {
        ctx.logger().info("Start native service");
        //        Err(merrors::Error::InternalServiceError("some internal error happened".to_string()))

        let value = ctx.env("CUSTOM_ENV").unwrap();
        ctx.logger().info(format!("env value '{}'", value).as_str());

        simple_api::execute_on(ctx, |api| {
            api.do_something();
            Ok(())
        })
        .await?;

        example::execute_on(ctx, |api| {
            api.do_something();
            Ok(())
        })
        .await?;

        ctx.logger().info("finished start native service method");
        Ok(())
    }

    async fn stop(&self, ctx: &Context) {
        ctx.logger().info("Stop native service");
    }
}
