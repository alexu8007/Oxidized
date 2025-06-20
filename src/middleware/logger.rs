use crate::{
    http_request::Request,
    middleware::Layer,
    service::Service,
    Error,
    Response,
    Result,
};
use std::future::Future;
use std::pin::Pin;

#[derive(Clone)]
pub struct LogLayer;

impl<S> Layer<S> for LogLayer {
    type Service = LogService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LogService { inner }
    }
}

#[derive(Clone)]
pub struct LogService<S> {
    inner: S,
}

impl<S> Service<Request> for LogService<S>
where
    S: Service<Request, Response = Response, Error = Error> + Clone + Send + Sync + 'static,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response>> + Send>>;

    fn call(&self, req: Request) -> Self::Future {
        let inner = self.inner.clone();
        Box::pin(async move {
            let method = req.inner().method().clone();
            let uri = req.inner().uri().clone();

            println!("request: {} {}", method, uri);

            inner.call(req).await
        })
    }
} 