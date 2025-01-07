use std::sync::Arc;

pub mod helloworld {
    include!("generated/helloworld.rs"); // Include generated code
}

use mikros::service::{builder::ServiceBuilder, context, lifecycle};
use mikros::link_grpc_service;
use tonic::transport::Channel;
use tonic::{Request, Response, Status};

use crate::helloworld::greeter_client::GreeterClient;
use helloworld::greeter_server::{Greeter, GreeterServer};
use helloworld::{HelloReply, HelloRequest};

#[derive(Clone)]
pub struct MyGreeter {
    ctx: Arc<mikros::Mutex<Context>>,
}

impl MyGreeter {
    pub fn new(ctx: Arc<mikros::Mutex<Context>>) -> Self {
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

        let reply = helloworld::HelloRequest::default();
        self.ctx
            .lock()
            .await
            .greeter
            .clone()
            .unwrap()
            .say_hello(reply)
            .await?;

        let _ = example::execute_on(ctx.clone(), |api| {
            api.do_something();
            Ok(())
        })
        .await;

        let reply = HelloReply {
            message: format!("Hello, {}!", request.into_inner().name),
        };

        self.ctx.lock().await.value += 1;
        Ok(Response::new(reply))
    }
}

#[derive(Clone, Default)]
pub struct Context {
    value: i32,
    greeter: Option<GreeterClient<Channel>>,
}

#[tonic::async_trait]
impl lifecycle::Lifecycle for Context {
    async fn on_start(&mut self, ctx: Arc<context::Context>) -> mikros::errors::Result<()> {
        println!("grpc on_start called");
        self.value = 42;
        self.greeter = Some(link_grpc_service!(ctx, GreeterClient, "greeter"));
        Ok(())
    }

    async fn on_finish(&self) -> mikros::errors::Result<()> {
        println!("grpc on_finish called");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Arc::new(mikros::Mutex::new(Context::default()));
    let greeter = Arc::new(MyGreeter::new(ctx.clone()));
    let greeter_service = GreeterServer::from_arc(greeter);

    let mut svc = ServiceBuilder::new()
        .grpc_with_lifecycle(greeter_service, ctx.clone())
        .build()?;

    Ok(svc.start().await?)
}
