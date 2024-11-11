mod service;

extern crate mikros;

use std::sync::{Arc, Mutex};

use mikros::service::builder::{ServiceBuilder};
use mikros::service::script::ScriptService;
use service::Service as AppService;

#[tokio::main]
async fn main() {
    let s = Arc::new(Mutex::new(AppService::new()));
    let svc = ServiceBuilder::default()
        .as_native(s.clone())
        .as_script(s)
        .build();

    match svc {
        Ok(mut svc) => {
            let _ = svc.start().await;
        },
        Err(e) => panic!("{}", e.to_string())
    }
}
