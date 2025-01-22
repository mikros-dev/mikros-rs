# Native services

A Native Service is a general-purpose application designed to execute any
arbitrary code for extended periods. It does not rely on a defined API and
is well-suited for tasks requiring flexible execution models.

Native services implement the [NativeService](../src/service/native/mod.rs)
trait to conform to the expected lifecycle and operational structure.

This trait ensures the service adheres to a standard lifecycle and
can be managed effectively within the broader application framework. It
is composed of the following methods:

- start: the place where the application is initialized and the main task
is executed.
- stop: the place to finish everything that was initialized before.

For more details about how to implement a native service check the following
[example](../../examples/apps/native).
