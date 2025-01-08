mod service;

use mikros::service::builder::ServiceBuilder;
use mikros::tokio;

use service::Service as AppService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let s = AppService::new();
    let mut svc = ServiceBuilder::default()
        .native(Box::new(s))
        .with_features(vec![simple_api::new(), example::new()])
        .build()?;

    Ok(svc.start().await?)
}
