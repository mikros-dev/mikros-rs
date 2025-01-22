# mikros

## Introduction

Mikros is a rust framework to help writing services or applications following the
same code structure and layout. It makes these applications demonstrate the same
behavior and results. It requires that they be built following the same pattern.

This way, it is possible to simply focus on the real problem that these applications
are trying to solve and stop worrying about their layout.

## Which problem mikros try to solve

As stated earlier, the framework aims to remove all the complexity involved in
building the structure of an application. Such as how and where to initialize
dependent resources, structures for passing data, etc.

Mikros imposes a format for initializing an application where the developer
determines its category and its main functionalities. From there, the pieces
just need to be fitted together, whether it is the handling of an RPC call
from a gRPC service or a task to be executed by an HTTP service when a
certain endpoint is triggered.

## Usage

Mikros is a rust crate, i.e., to be used inside rust projects is just like any
common crate. One must add it as a dependency in the Cargo.toml file and its
API should be available to be used.

To work properly, the following steps can be used to write a new application
with it:

- create a `service.toml` file defining the service kind of the application;
- implement the trait related to the chosen service kind;
- initialize the API using the service implementation.

Example: suppose you need an application that needs to run for an indefinite
period of time, performing some type of task that cannot be stopped. For this,
you can choose the **native** service kind to be used.

First, create a new cargo project for binary applications:

```bash
cargo new native-service
```

Then, inside the new service directory, create the service definitions file.
This is how it should look like:

```toml
name = "my-service-name"
types = [ "native" ]
version = "v0.1.0"
language = "rust"
product = "my-awesome-product"
```

Adjust the required dependencies inside the project **Cargo.toml** file by
adding the following:

- async-trait
- mikros
- tokio

Inside the **main.rs** file, implement the following:

- a structure that will implement the NativeService trait:

```rust
#[derive(Clone, Default)]
pub struct AppService;

#[async_trait::async_trait]
impl mikros::service::native::NativeService for AppService {
    async fn start(&self, ctx: &mikros::service::context::Context) -> mikros::errors::Result<()> {
        // ctx is the mechanism to access mikros APIs and get the logger,
        // environment variables, service definitions and access to features.
        Ok(())
    }

    async fn stop(&self, ctx: &Context) {
        // release some resource that the application is holding
    }
}
```

- if required to initialize some resource for the service, implement the Lifecycle
trait:

```rust
#[async_trait::async_trait]
impl mikros::service::lifecycle::Lifecycle for AppService {
    async fn on_start(&mut self, ctx: &mikros::service::context::Context) -> mikros::errors::Result<()> {
        // again, ctx can help access the mikros API and use it for initialize
        // some custom internal service resource.
        Ok(())
    }

    async fn on_finish(&self) -> mikros::errors::Result<()> {
        Ok(())
    }
}
```

- initialize the service using mikros API:

```rust
#[tokio::main]
async fn main() {
    let s = AppService::default();
    let svc = mikros::service::builder::ServiceBuilder::default()
        .native(Box::new(s))
        .build();

    match svc {
        Ok(mut svc) => svc.start().await,
        Err(e) => panic!("{}", e.to_string()),
    }
}
```

The complete example can be found inside the [examples](../../examples/apps/native) directory.

### Service definitions file

The service definitions file is a [TOML](https://toml.io/en/) file with information
about the service.

Its objective is, in addition to defining some important information for executing
the application, some other information to assist possible deployment or monitoring
tools.

Its mandatory content is summarized in the following fields:

- name: a string to set the application name
- types: a string array defining the service kind, which can be more than one,
as long as they have the same mode of execution (Block/NonBlock)
- version: the application version
- language: the programming language of the application
- product: the product name to which the application belongs

Optional fields:

- envs: a string array of required environment variables for the service.
- log: an object that allows defining the service initial settings for logging.

Additionally, you can use this same file for the following types of definitions:

- features: a map of feature definitions, to set custom features settings for
the application.
- services: a map of service definitions, to set custom service kind settings
for the application.
- clients: a map of client connection definitions, for dependent applications.
- service: a structure free service definition that the service can use for
its own settings.

Example:

```toml
name = "my-service-name"
types = [ "native" ]
version = "v0.1.0"
language = "rust"
product = "my-awesome-product"
envs = [ "CUSTOM_ENV_1" ]

# Required settings for the simple_api feature
[features.simple_api]
enabled = true

# Custom settings for connecting with the 'grpc' gRPC client
[clients.grpc]
host = "localhost"
port = 7071

# Required settings for the 'cronjob' service type
[services.cronjob]
frequency = "daily"

# Custom service settings
[service]
max_diff_range = 100
```

### Environment variables

Mikros has some environment variables that it uses to set custom information
while the application is running. They are the following:

| Name                        | Description                                                                                                              |
|-----------------------------|--------------------------------------------------------------------------------------------------------------------------|
| MIKROS_SERVICE_DEPLOY       | A string to set the current deployment server of the application, like (dev, stage, prod). Default: local                |
| MIKROS_TRACKER_HEADER_NAME  | A header name where the track ID will be located in HTTP requests/responses (not implemented yet). Default: X-Request-ID |
| MIKROS_COUPLED_NAMESPACE    | The namespace where services/applications are running, to build the gRPC connection URL. Default: localhost              | 
| MIKROS_COUPLED_PORT         | The default port for dependent gRPC services. Default: 7070                                                              |
| MIKROS_GRPC_PORT            | Default listening port for gRPC applications. Default: 7070                                                              |
| MIKROS_HTTP_PORT            | Default listening port for HTTP applications. Default: 8080                                                              |
| MIKROS_HIDE_RESPONSE_FIELDS | A comma separated list of fields to be hidden in HTTP services error response.                                           |

### The service structure

Each service kind has its own trait that needs to be implemented in the application
main structure and passed in the API initialization.

A gRPC application depends on the declared API in the protobuf file. So the
application must implement it.

An HTTP application instead does not have a proper trait or API to implement. It
depends on its endpoints, where each one should have a handler for it.

### Lifecycle

Lifecycle is a trait that an application can implement in its main structure to
receive callbacks from the framework at specific execution points. Allowing it
to handle them to execute custom tasks at these points. Like for example
initialize connections with coupled gRPC services that it requires.

Its declaration is the following:

```rust
#[async_trait::async_trait]
pub trait Lifecycle: LifecycleClone + Send + Sync {
    async fn on_start(&mut self, _ctx: &mikros::service::context::Context) -> mikros::errors::Result<()> {
        Ok(())
    }

    async fn on_finish(&self) -> merrors::Result<()> {
        Ok(())
    }
}
```

Notice that the trait already has default implementation, so the service can
choose not to implement it.

### Extending features

Mikros does not provide any feature out-of-the-box, like cache, database and
similar stuff. But it provides an API to allow implementing support for these
and to access them using the same syntax inside applications.

The trait [Feature](../src/plugin/feature.rs) shows an API that a feature
should implement to be handled inside by the framework and provided to the
applications. It has methods with the following characteristics:

- provide the feature name and information to be registered while the application
is initializing.
- initialize itself and clean its resources.
- a public API for applications to use it.

For an example of how to implement, register and use external features you can
check the [features](../../examples/features) examples directory.

### Extending service kind

As mentioned before, mikros provides some kind of services that it implements
inside and allows building applications with them. But if a specific kind of
application (or service) is required by some solution, it also provides an API
to allow implementing and to allow new kind of applications being built.

The trait [Service](../src/plugin/service.rs) shows an API that a new service
kind should implement to be supported by the framework, and, consequently, new
applications.

This API requires that the implementation provides information about the new
service type such as:

- its name and information to be registered while the application is initializing.
- its execution mode (blocking or non-blocking).
- and its implementation.

The [services](../../examples/services) directory shows an example of how to create
a new service kind and the [cronjob](../../examples/apps/cronjob_service) service
shows how to register it and use it.
