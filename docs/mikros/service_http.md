# HTTP services

An HTTP Service is a web-based service designed to define and handle HTTP
endpoints and routes. These services use the [axum](https://docs.rs/axum/latest/axum/)
web framework to build flexible and efficient APIs.

A mikros HTTP service requires that each router handler should have, at least,
a [State](https://docs.rs/axum/latest/axum/#using-the-state-extractor) in its
arguments. Like the following:

```rust
use mikros::http::ServiceState;

async fn handler(State(state): State<Arc<Mutex<ServiceState>>>) -> String {
    // state will provide access to the framework context, and if chosen, to
    // a custom application internal state.
    format!("Handler")
}
```

The examples application directory contains different examples of how to implement
an HTTP service using mikros.
