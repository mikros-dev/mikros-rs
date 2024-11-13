// src/server.rs
use tonic::{transport::Server, Request, Response, Status};
use helloworld::greeter_server::{Greeter, GreeterServer};
use helloworld::{HelloRequest, HelloReply};

pub mod helloworld {
    include!("generated/helloworld.rs"); // Include generated code
}

#[derive(Default, Clone)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(&self, request: Request<HelloRequest>) -> Result<Response<HelloReply>, Status> {
        let reply = HelloReply {
            message: format!("Hello, {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let greeter = MyGreeter::default();
    let greeter_service = GreeterServer::new(greeter);

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(greeter_service)
        .serve(addr)
        .await?;

    Ok(())
}
