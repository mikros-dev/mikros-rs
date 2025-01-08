use std::sync::Arc;

use cronjob::builder::CronjobBuilder;
use mikros::service::builder::ServiceBuilder;
use mikros::{async_trait, tokio, errors};

#[derive(Default)]
pub struct Service;

#[async_trait::async_trait]
impl cronjob::CronjobService for Service {
    async fn handler(&mut self, ctx: Arc<mikros::service::context::Context>) -> errors::Result<()> {
        ctx.logger().info("handler executed");

        simple_api::execute_on(ctx.clone(), |api| {
            api.do_something();
            Ok(())
        }).await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let s = Box::new(Service::default());
    let c = Box::new(CronjobBuilder::new(s).build());

    let mut svc = ServiceBuilder::new()
        .custom(c)
        .with_features(vec![simple_api::new(), example::new()])
        .build()?;

    Ok(svc.start().await?)
}
