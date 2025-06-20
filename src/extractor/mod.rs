use crate::{
    http_request::{Body, RequestParts},
    Error, Result,
};
use async_trait::async_trait;
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;

#[async_trait]
pub trait FromRequest: Sized {
    async fn from_request(parts: &mut RequestParts) -> Result<Self>;
}

#[async_trait]
pub trait FromBody: Sized {
    async fn from_body(body: Body) -> Result<Self>;
}

pub struct Json<T>(pub T);

#[async_trait]
impl<T> FromBody for Json<T>
where
    T: DeserializeOwned + Send,
{
    async fn from_body(body: Body) -> Result<Self> {
        let body_bytes = body.collect().await.map_err(|_| Error::NotFound)?.to_bytes();
        let data = serde_json::from_slice(&body_bytes).map_err(|_| Error::NotFound)?;
        Ok(Json(data))
    }
}

#[async_trait]
impl FromBody for String {
    async fn from_body(body: Body) -> Result<Self> {
        let body_bytes = body.collect().await.map_err(|_| Error::NotFound)?.to_bytes();
        String::from_utf8(body_bytes.to_vec()).map_err(|_| Error::NotFound)
    }
} 