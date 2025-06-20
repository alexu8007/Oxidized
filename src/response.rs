use http::Response as HttpResponse;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::header::HeaderValue;
use hyper::StatusCode;

pub struct Response {
    inner: HttpResponse<Full<Bytes>>,
}

impl Response {
    pub fn new<T: Into<Bytes>>(body: T) -> Self {
        Self {
            inner: HttpResponse::new(Full::new(body.into())),
        }
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        *self.inner.status_mut() = status;
        self
    }

    pub fn header(mut self, key: &'static str, value: &'static str) -> Self {
        self.inner
            .headers_mut()
            .insert(key, HeaderValue::from_static(value));
        self
    }

    pub fn into_hyper(self) -> HttpResponse<Full<Bytes>> {
        self.inner
    }

    pub(crate) fn inner_mut(&mut self) -> &mut HttpResponse<Full<Bytes>> {
        &mut self.inner
    }
}

impl From<&'static str> for Response {
    fn from(body: &'static str) -> Self {
        Response::new(body)
    }
}

impl From<String> for Response {
    fn from(body: String) -> Self {
        Response::new(body)
    }
}

impl From<Bytes> for Response {
    fn from(body: Bytes) -> Self {
        Response::new(body)
    }
} 