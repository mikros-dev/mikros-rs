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
impl mikros::service::script::ScriptService for Service {
    async fn run(&self, ctx: Arc<Context>) -> errors::Result<()> {
        ctx.logger().info("Start script service");
        example::execute_on(ctx, |api| {
            api.do_something();
            Ok(())
        })
        .await?;

        Ok(())
    }

    async fn cleanup(&self, ctx: Arc<Context>) {
        ctx.logger().info("Stop script service");
    }
}
