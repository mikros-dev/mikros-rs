mod service;

use mikros::service::builder::ServiceBuilder;
use mikros::tokio;

use service::Service as AppService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let s = AppService::new();
    let mut svc = ServiceBuilder::new().grpc(s).build()?;

    Ok(svc.start().await?)
}
