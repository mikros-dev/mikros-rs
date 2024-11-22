![status](resources/badges/status.svg)
![coverage](resources/badges/coverage.svg)
![license](resources/badges/license.svg)

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

* [gRPC](docs/service_grpc.md): an application with an API defined from a [protobuf](https://protobuf.dev) file.
* [HTTP](docs/service_http.md): an HTTP server-type application.
* [native](docs/service_native.md): a general-purpose application, without a defined API, with the ability to execute any code for long periods
* [script](docs/service_script.md): also a general-purpose application, without a defined API, but that only needs to execute a single function and stop.

### Documentation

A more detailed documentation about the framework API and its features can be
accessed at the [docs](docs/mikros.md) directory.

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
version = "v0.1.0"
language = "rust"
product = "Matrix"
```
When executed, it outputs the following (with a different time according the execution):

```bash
2024-11-21 22:08:04.415725 -03:00	INFO	service starting	{"local.ts":1732237684,"svc.version":"v0.1.0","svc.language":"rust","local.ts_ms":1732237684415,"svc.name":"script-example","svc.product":"Matrix"}
2024-11-21 22:08:04.416153 -03:00	INFO	starting features	{"local.ts":1732237684,"svc.version":"v0.1.0","svc.language":"rust","local.ts_ms":1732237684416,"svc.name":"script-example","svc.product":"Matrix"}
lifecycle on_start
2024-11-21 22:08:04.416174 -03:00	INFO	service resources	{"local.ts":1732237684,"svc.version":"v0.1.0","svc.language":"rust","local.ts_ms":1732237684416,"svc.name":"script-example","svc.product":"Matrix"}
2024-11-21 22:08:04.416194 -03:00	INFO	service is running	{"local.ts":1732237684,"svc.version":"v0.1.0","svc.language":"rust","local.ts_ms":1732237684416,"kind":"script","svc.name":"script-example","svc.product":"Matrix"}
2024-11-21 22:08:04.416218 -03:00	INFO	Stop script service	{"local.ts":1732237684,"svc.version":"v0.1.0","svc.language":"rust","local.ts_ms":1732237684416,"svc.name":"script-example","svc.product":"Matrix"}
2024-11-21 22:08:04.416234 -03:00	DEBUG	sending shutdown signal for service tasks	{"local.ts":1732237684,"svc.version":"v0.1.0","svc.language":"rust","local.ts_ms":1732237684416,"svc.name":"script-example","svc.product":"Matrix"}
2024-11-21 22:08:04.416250 -03:00	DEBUG	starting service task	{"local.ts":1732237684,"svc.version":"v0.1.0","svc.language":"rust","local.ts_ms":1732237684416,"task_name":"script","svc.name":"script-example","svc.product":"Matrix"}
2024-11-21 22:08:04.416351 -03:00	INFO	Start script service	{"local.ts":1732237684,"svc.version":"v0.1.0","svc.language":"rust","local.ts_ms":1732237684416,"svc.name":"script-example","svc.product":"Matrix"}
2024-11-21 22:08:04.416370 -03:00	DEBUG	finishing service task	{"local.ts":1732237684,"svc.version":"v0.1.0","svc.language":"rust","local.ts_ms":1732237684416,"task_name":"script","svc.name":"script-example","svc.product":"Matrix"}
2024-11-21 22:08:04.416388 -03:00	DEBUG	service task finished	{"local.ts":1732237684,"svc.version":"v0.1.0","svc.language":"rust","local.ts_ms":1732237684416,"task_name":"script","svc.name":"script-example","svc.product":"Matrix"}
lifecycle on_finish
2024-11-21 22:08:04.416436 -03:00	INFO	service stopped	{"local.ts":1732237684,"svc.version":"v0.1.0","svc.language":"rust","local.ts_ms":1732237684416,"svc.name":"script-example","svc.product":"Matrix"}
```

For more examples, including how to create new features and new service kind,
you can check the [examples](examples) directory.

## Roadmap

* Improve logger to use better structured log system and to set the log level
using service definitions.
* Enable features and services to use the same env system that the core uses.
* Improve unit tests.
* Full compatibility between go and rust gRPC services.
* Allow using the service definitions file for custom service definitions.

## License

[MIT License](LICENSE)
