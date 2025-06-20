use crate::{
    extractor::FromBody,
    middleware::{Layer, Stack},
    Error, Request, Response, Result, Service,
    ws::upgrade::upgrade as ws_upgrade,
};
use async_trait::async_trait;
use http::Method;
use std::{collections::HashMap, future::Future, marker::Send, pin::Pin, sync::Arc};

#[async_trait]
pub trait Handler<Args>: Clone + Send + Sync + 'static {
    async fn call(self, req: Request) -> Result<Response>;
}

#[async_trait]
impl<F, Fut> Handler<()> for F
where
    F: Fn() -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<Response>> + Send,
{
    async fn call(self, _req: Request) -> Result<Response> {
        self().await
    }
}

#[async_trait]
impl<F, Fut, T> Handler<(T,)> for F
where
    F: Fn(T) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = Result<Response>> + Send,
    T: FromBody + Send,
{
    async fn call(self, req: Request) -> Result<Response> {
        let (_parts, body) = req.into_parts();
        let arg = T::from_body(body).await?;
        self(arg).await
    }
}

type BoxedHandler =
    Arc<dyn Fn(Request) -> Pin<Box<dyn Future<Output = Result<Response>> + Send>> + Send + Sync>;
type WsHandler = Box<dyn Fn(Request) -> Result<Response> + Send + Sync>;

#[derive(Clone)]
pub struct Router {
    routes: HashMap<Method, HashMap<String, BoxedHandler>>,
    ws_routes: HashMap<String, Arc<WsHandler>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
            ws_routes: HashMap::new(),
        }
    }

    pub fn get<F, Args>(mut self, path: &str, handler: F) -> Self
    where
        F: Handler<Args> + 'static,
        Args: Send + 'static,
    {
        let handler: BoxedHandler = Arc::new(move |req: Request| {
            let handler = handler.clone();
            Box::pin(handler.call(req))
        });

        let entry = self.routes.entry(Method::GET).or_default();
        entry.insert(path.to_string(), handler);
        self
    }

    pub fn ws<F, Fut>(mut self, path: &str, f: F) -> Self
    where
        F: Fn(crate::ws::WebSocket) -> Fut + Send + Sync + 'static + Clone,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.ws_routes
            .insert(path.to_string(), Arc::new(Box::new(ws_upgrade(f))));
        self
    }

    pub fn layer<L>(self, layer: L) -> Stack<L, Self>
    where
        L: Layer<Self>,
    {
        Stack::new(layer, self)
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Service<Request> for Router {
    type Response = Response;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response>> + Send>>;

    fn call(&self, req: Request) -> Self::Future {
        let routes = self.routes.clone();
        let ws_routes = self.ws_routes.clone();
        Box::pin(async move {
            let method = req.inner().method();
            let path = req.inner().uri().path();

            if let Some(handler) = ws_routes.get(path) {
                return handler(req);
            }

            if let Some(routes) = routes.get(method) {
                if let Some(handler) = routes.get(path) {
                    return handler(req).await;
                }
            }
            Err(Error::NotFound)
        })
    }
}