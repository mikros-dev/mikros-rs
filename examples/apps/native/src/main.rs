mod service;

use mikros::service::builder::ServiceBuilder;
use service::Service as AppService;

#[tokio::main]
async fn main() {
    let s = AppService::new();
    let svc = ServiceBuilder::default()
        .native(Box::new(s))
        .with_features(simple_api::features())
        .build();

    match svc {
        Ok(mut svc) => {
            if let Err(e) = svc.start().await {
                println!("application error: {}", e);
            }
        }
        Err(e) => panic!("{}", e.to_string()),
    }
}