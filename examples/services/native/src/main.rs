mod service;

extern crate mikros;

use std::sync::Arc;

use mikros::FutureMutex;
use mikros::service::builder::{ServiceBuilder};
use service::Service as AppService;

#[tokio::main]
async fn main() {
    let s = Arc::new(FutureMutex::new(AppService::new()));
    let svc = ServiceBuilder::default()
        .script(s.clone())
        .native(s.clone())
        .build();

    match svc {
        Ok(mut svc) => {
            if let Err(e) = svc.start().await {
                println!("application error: {}", e);
            }
        },
        Err(e) => panic!("{}", e.to_string())
    }
}
