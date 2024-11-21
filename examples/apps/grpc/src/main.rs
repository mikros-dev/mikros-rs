use std::sync::Arc;

pub mod helloworld {
    include!("generated/helloworld.rs"); // Include generated code
}

use mikros::features;
use mikros::service::{builder::ServiceBuilder, context};
use tonic::{Request, Response, Status};

use helloworld::greeter_server::{Greeter, GreeterServer};
use helloworld::{HelloReply, HelloRequest};

#[derive(Clone)]
pub struct MyGreeter {
    ctx: Arc<mikros::FutureMutex<Context>>,
}

impl MyGreeter {
    pub fn new(ctx: Arc<mikros::FutureMutex<Context>>) -> Self {
        Self { ctx: ctx.clone() }
    }
}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let ctx = context::from_request(&request);
        ctx.logger().info("say hello RPC called");
        ctx.logger()
            .info(format!("the inner value is: {}", self.ctx.lock().await.value).as_str());

        let _ = features::example::retrieve(&ctx, |api| {
            api.do_something();
        });

        let reply = HelloReply {
            message: format!("Hello, {}!", request.into_inner().name),
        };

        self.ctx.lock().await.value += 1;
        Ok(Response::new(reply))
    }
}

#[derive(Clone)]
pub struct Context {
    value: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Arc::new(mikros::FutureMutex::new(Context { value: 0 }));
    let greeter = Arc::new(MyGreeter::new(ctx.clone()));
    let greeter_service = GreeterServer::from_arc(greeter);

    let svc = ServiceBuilder::default().grpc(greeter_service).build();

    match svc {
        Ok(mut svc) => {
            if let Err(e) = svc.start().await {
                println!("application error: {}", e);
            }
        }
        Err(e) => panic!("{}", e.to_string()),
    }

    Ok(())
}
