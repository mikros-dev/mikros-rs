# gRPC services

The gRPC service kind supported by the framework is the way to build applications
capable of using the [gRPC](https://grpc.io) framework behind the scenes.

This kind of application requires that its API to be defined using a [protobuf](https://protobuf.dev)
spec file.

Mikros uses [tonic](https://github.com/hyperium/tonic) rust gRPC implementation
internally.

In order to create a new service of this kind, the following steps can be used
to help:

- create the service protobuf file with its API.
- inside the project directory, use [tonic_build](https://docs.rs/tonic-build/latest/tonic_build/)
and with a **build.rs** source file, compile the protobuf to generate rust code
from it.
- implement the service by implementing its API in a structure and use mikros
API to initialize the service.

The examples directory has the following examples using the same steps described:

* [grpc](../../examples/apps/grpc): a gRPC service which implements its API.
* [grpc with lifecycle](../../examples/apps/grpc_with_lifecycle): a gRPC service
that also implements the Lifecycle trait.
