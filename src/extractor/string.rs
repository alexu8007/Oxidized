use crate::{
    extractor::FromRequest,
    http_request::{Body, RequestParts},
    Error, Result,
};
use async_trait::async_trait;
use http_body_util::BodyExt;

#[async_trait]
impl FromRequest for String {
    async fn from_request(_parts: &mut RequestParts, body: Body) -> Result<Self> {
        let body_bytes = body.collect().await.map_err(|_| Error::NotFound)?.to_bytes();
        String::from_utf8(body_bytes.to_vec()).map_err(|_| Error::NotFound)
    }
} 