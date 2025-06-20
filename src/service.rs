use std::future::Future;

pub trait Service<Req> {
    type Response;
    type Error;
    type Future: Future<Output = Result<Self::Response, Self::Error>>;

    fn call(&self, req: Req) -> Self::Future;
}

pub fn service_fn<F, Req, Res, E>(f: F) -> ServiceFn<F>
where
    F: Fn(Req) -> <ServiceFn<F> as Service<Req>>::Future,
    ServiceFn<F>: Service<Req, Response = Res, Error = E>,
{
    ServiceFn { f }
}

#[derive(Clone, Copy)]
pub struct ServiceFn<F> {
    f: F,
}

impl<F, Fut, Req, Res, E> Service<Req> for ServiceFn<F>
where
    F: Fn(Req) -> Fut,
    Fut: Future<Output = Result<Res, E>>,
{
    type Response = Res;
    type Error = E;
    type Future = Fut;

    fn call(&self, req: Req) -> Self::Future {
        (self.f)(req)
    }
} 