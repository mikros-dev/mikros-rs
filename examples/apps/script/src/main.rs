mod service;

use mikros::service::builder::ServiceBuilder;
use service::Service as AppService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let s = AppService::new();
    let mut svc = ServiceBuilder::default()
        .script(Box::new(s))
        .with_features(vec![example::new()])
        .build()?;

    Ok(svc.start().await?)
}
