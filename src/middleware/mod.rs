pub mod logger;

pub use self::logger::LogLayer;
use crate::service::Service;
use std::sync::Arc;

pub trait Layer<S> {
    type Service;

    fn layer(&self, inner: S) -> Self::Service;
}

#[derive(Clone)]
pub struct Stack<L, S> {
    layer: L,
    service: S,
}

impl<L, S> Stack<L, S> {
    pub fn new(layer: L, service: S) -> Self {
        Self { layer, service }
    }
}

impl<L, S, Req> Service<Req> for Stack<L, S>
where
    L: Layer<S>,
    S: Clone,
    L::Service: Service<Req>,
{
    type Response = <L::Service as Service<Req>>::Response;
    type Error = <L::Service as Service<Req>>::Error;
    type Future = <L::Service as Service<Req>>::Future;

    fn call(&self, req: Req) -> Self::Future {
        let s = self.layer.layer(self.service.clone());
        s.call(req)
    }
}

#[derive(Clone)]
pub struct ArcLayer<S>(Arc<S>);

impl<S> ArcLayer<S> {
    pub fn new(service: S) -> Self {
        Self(Arc::new(service))
    }
}

impl<S> Layer<S> for ArcLayer<S>
where
    S: Service<crate::Request>,
{
    type Service = Arc<S>;

    fn layer(&self, _inner: S) -> Self::Service {
        self.0.clone()
    }
} 