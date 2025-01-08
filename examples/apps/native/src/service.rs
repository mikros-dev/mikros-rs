use std::sync::Arc;

use mikros::{async_trait, errors};
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
    async fn on_start(&mut self, _ctx: Arc<Context>) -> errors::Result<()> {
        println!("lifecycle on_start");
        Ok(())
    }

    async fn on_finish(&self) -> errors::Result<()> {
        println!("lifecycle on_finish");
        Ok(())
    }
}

#[async_trait::async_trait]
impl mikros::service::native::NativeService for Service {
    async fn start(&self, ctx: Arc<Context>) -> errors::Result<()> {
        ctx.logger().info("Start native service");
        //        Err(merrors::Error::InternalServiceError("some internal error happened".to_string()))

        let value = ctx.env("CUSTOM_ENV").unwrap();
        ctx.logger().info(format!("env value '{}'", value).as_str());

        simple_api::execute_on(ctx.clone(), |api| {
            api.do_something();
            Ok(())
        })
        .await?;

        example::execute_on(ctx.clone(), |api| {
            api.do_something();
            Ok(())
        })
        .await?;

        ctx.logger().info("finished start native service method");
        Ok(())
    }

    async fn stop(&self, ctx: Arc<Context>) {
        ctx.logger().info("Stop native service");
    }
}
