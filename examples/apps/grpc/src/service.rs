
use std::sync::Arc;

pub mod helloworld {
    include!("generated/helloworld.rs"); // Include generated code
}

use mikros::service::context;
use tonic::{Request, Response, Status};

use helloworld::greeter_server::{Greeter, GreeterServer};
use helloworld::{HelloReply, HelloRequest};

#[derive(Clone)]
pub struct Context {
    value: i32,
}

#[derive(Clone)]
pub struct Service {
    ctx: Arc<mikros::Mutex<Context>>,
}

impl Service {
    pub fn new() -> GreeterServer<Service> {
        let ctx = Arc::new(mikros::Mutex::new(Context { value: 0 }));
        let greeter = Arc::new(Self::new_service(ctx.clone()));

        GreeterServer::from_arc(greeter)
    }

    fn new_service(ctx: Arc<mikros::Mutex<Context>>) -> Self {
        Self { ctx: ctx.clone() }
    }
}

#[tonic::async_trait]
impl Greeter for Service {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let ctx = context::from_request(&request);
        ctx.logger().info("say hello RPC called");
        ctx.logger()
            .info(format!("the inner value is: {}", self.ctx.lock().await.value).as_str());

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
