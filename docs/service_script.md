# Script services

A script service is a general-purpose application designed for one-off tasks
that execute a single function and then terminate. Unlike continuous services,
script services are concise, focusing on completing a specific operation and
performing any necessary cleanup afterward.

To be a script service it must implement the [ScriptService](../src/service/script/mod.rs)
trait to standardize their lifecycle behavior.

For more details about how to implement a script service check the following
[example](../examples/apps/script).
