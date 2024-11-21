# mikros-rs

## About

`mikros` is rust framework for creating applications.

### Introduction

Mikros is an opinionated framework aiming to ease and standardize the creation
of applications. It supports creating applications that need to run for long
periods, usually executing indefinitely, performing some specific operation or
providing some API to be consumed by other services. But it also supports
standalone applications that execute its task and finishes.

Currently, it has support for the following kind of applications:

* gRPC: an application with an API defined from a [protobuf](https://protobuf.dev) file.
* HTTP: an HTTP server-type application.
* native: a general-purpose application, without a defined API, with the ability to execute any code for long periods
* script: also a general-purpose application, without a defined API, but that only needs to execute a single function and stop.

### Service

Service, here, is considered an application that may or may not remain running
indefinitely, performing some type of task or waiting for commands to activate it.

The framework consists of an SDK that facilitates the creation of these applications
in a way that standardizes their code, so that they all perform tasks with the
same behavior and are written in a very similar manner. In addition to providing
flexibility, allowing these applications to also be customized when necessary.

Building a service using the framework's SDK must adhere to the following points:

* Have a struct where mandatory methods according to its category must be implemented;
* Initialize the SDK correctly;
* Have a configuration file, called `service.toml`, containing information about itself and its functionalities.

### Example of a service

The following example demonstrates how to create a service of a `script`
type. The `Service` structure implements the trait [ScriptService](src/service/script/mod.rs)
that makes it being supported by this type of service inside the framework.

```rust
use mikros::errors as merrors;
use mikros::service::builder::ServiceBuilder;
use mikros::service::context::Context;

#[derive(Clone, Default)]
pub struct Service;

#[async_trait::async_trait]
impl mikros::service::lifecycle::Lifecycle for Service {
    async fn on_start(&mut self, _ctx: &Context) -> merrors::Result<()> {
        println!("lifecycle on_start");
        Ok(())
    }

    async fn on_finish(&self) -> merrors::Result<()> {
        println!("lifecycle on_finish");
        Ok(())
    }
}

#[async_trait::async_trait]
impl mikros::service::script::ScriptService for Service {
    async fn run(&self, ctx: &Context) -> merrors::Result<()> {
        ctx.logger().info("Start script service");
        Ok(())
    }

    async fn cleanup(&self, ctx: &Context) {
        ctx.logger().info("Stop script service");
    }
}

#[tokio::main]
async fn main() {
    let s = Service::default();
    let svc = ServiceBuilder::default()
        .script(Box::new(s))
        .build();

    match svc {
        Ok(mut svc) => svc.start().await,
        Err(e) => panic!("{}", e.to_string()),
    }
}
```

It must have a `service.toml` file with the following content:

```toml
name = "script-example"
types = ["script"]
version = "v1.0.0"
language = "rust"
product = "Matrix"
```
When executed, it outputs the following (with a different time according the execution):

```bash
2024-11-21 20:24:50.215638 -03:00	INFO	service starting	{"svc.language":"rust","local.ts":1732231490,"local.ts_ms":1732231490215,"svc.name":"native-example","svc.product":"examples","svc.version":"v0.1.0"}
2024-11-21 20:24:50.215953 -03:00	INFO	starting features	{"svc.language":"rust","local.ts":1732231490,"local.ts_ms":1732231490215,"svc.name":"native-example","svc.product":"examples","svc.version":"v0.1.0"}
lifecycle on_start
2024-11-21 20:24:50.215974 -03:00	INFO	service resources	{"svc.language":"rust","local.ts":1732231490,"local.ts_ms":1732231490215,"svc.name":"native-example","svc.product":"examples","svc.version":"v0.1.0"}
2024-11-21 20:24:50.215993 -03:00	INFO	service is running	{"svc.language":"rust","local.ts":1732231490,"kind":"script","local.ts_ms":1732231490215,"svc.name":"native-example","svc.product":"examples","svc.version":"v0.1.0"}
2024-11-21 20:24:50.216023 -03:00	INFO	Stop script service	{"svc.language":"rust","local.ts":1732231490,"local.ts_ms":1732231490216,"svc.name":"native-example","svc.product":"examples","svc.version":"v0.1.0"}
2024-11-21 20:24:50.216043 -03:00	DEBUG	sending shutdown signal for service tasks	{"svc.language":"rust","local.ts":1732231490,"local.ts_ms":1732231490216,"svc.name":"native-example","svc.product":"examples","svc.version":"v0.1.0"}
2024-11-21 20:24:50.216043 -03:00	DEBUG	starting service task	{"svc.language":"rust","local.ts":1732231490,"task_name":"script","local.ts_ms":1732231490216,"svc.name":"native-example","svc.product":"examples","svc.version":"v0.1.0"}
2024-11-21 20:24:50.216148 -03:00	INFO	Start script service	{"svc.language":"rust","local.ts":1732231490,"local.ts_ms":1732231490216,"svc.name":"native-example","svc.product":"examples","svc.version":"v0.1.0"}
2024-11-21 20:24:50.216168 -03:00	DEBUG	finishing service task	{"svc.language":"rust","local.ts":1732231490,"task_name":"script","local.ts_ms":1732231490216,"svc.name":"native-example","svc.product":"examples","svc.version":"v0.1.0"}
2024-11-21 20:24:50.216186 -03:00	DEBUG	service task finished	{"svc.language":"rust","local.ts":1732231490,"task_name":"script","local.ts_ms":1732231490216,"svc.name":"native-example","svc.product":"examples","svc.version":"v0.1.0"}
lifecycle on_finish
2024-11-21 20:24:50.216234 -03:00	INFO	service stopped	{"svc.language":"rust","local.ts":1732231490,"local.ts_ms":1732231490216,"svc.name":"native-example","svc.product":"examples","svc.version":"v0.1.0"}
```

For more examples, including how to create new features and new service kind,
you can check the [examples](examples) directory.

## Roadmap

* Improve logger to use better structured log system and to set the log level
using service definitions.
* Enable features and services to use the same env system that the core uses.
* Improve unit tests.
* Full compatibility between go and rust gRPC services.

## License

[MIT License](LICENSE)
