use std::sync::Arc;
use std::task::{Context, Poll};

use tower::{Layer, Service};

use crate::service::context;

#[derive(Clone)]
pub(crate) struct ContextExtractor {
    ctx: Arc<context::Context>,
}

impl ContextExtractor {
    pub(crate) fn new(ctx: &context::Context) -> Self {
        ContextExtractor {
            ctx: Arc::new(ctx.clone()),
        }
    }
}

impl<S> Layer<S> for ContextExtractor {
    type Service = ContextExtractorMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        ContextExtractorMiddleware {
            inner: service,
            ctx: self.ctx.clone(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ContextExtractorMiddleware<S> {
    inner: S,
    ctx: Arc<context::Context>,
}

impl<S, B> Service<http::Request<B>> for ContextExtractorMiddleware<S>
where
    S: Service<http::Request<B>, Response = http::Response<B>>
        + Clone
        + Send
        + 'static,
    S::Future: Send,
    B: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: http::Request<B>) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        req.extensions_mut().insert(self.ctx.clone());
        Box::pin(async move {
            let response = inner.call(req).await?;
            Ok(response)
        })
    }
}
