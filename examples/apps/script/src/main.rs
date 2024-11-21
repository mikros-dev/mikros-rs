mod service;

use mikros::service::builder::ServiceBuilder;
use service::Service as AppService;

#[tokio::main]
async fn main() {
    let s = AppService::new();
    let svc = ServiceBuilder::default().script(Box::new(s)).build();

    match svc {
        Ok(mut svc) => svc.start().await,
        Err(e) => panic!("{}", e.to_string()),
    }
}
