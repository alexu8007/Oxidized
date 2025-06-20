use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::{HeaderMap, Method, Uri};

pub struct Request {
    inner: hyper::Request<Incoming>,
}

pub struct RequestParts {
    pub method: Method,
    pub uri: Uri,
    pub headers: HeaderMap,
}

pub type Body = hyper::body::Incoming;

impl Request {
    pub fn from_hyper(req: hyper::Request<Incoming>) -> Self {
        Self { inner: req }
    }

    pub fn into_parts(self) -> (RequestParts, Body) {
        let (parts, body) = self.inner.into_parts();
        let request_parts = RequestParts {
            method: parts.method,
            uri: parts.uri,
            headers: parts.headers,
        };
        (request_parts, body)
    }

    pub fn inner(&self) -> &hyper::Request<Incoming> {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut hyper::Request<Incoming> {
        &mut self.inner
    }

    pub async fn body_bytes(self) -> Result<bytes::Bytes, hyper::Error> {
        self.inner.into_body().collect().await.map(|body| body.to_bytes())
    }
} 