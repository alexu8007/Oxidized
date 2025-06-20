use crate::{
    extractor::FromRequest,
    http_request::{Body, RequestParts},
    Error, Request, Result,
};
use async_trait::async_trait;
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;

pub struct Json<T>(pub T);

#[async_trait]
impl<T> FromRequest for Json<T>
where
    T: DeserializeOwned + Send,
{
    async fn from_request(_parts: &mut RequestParts, body: Body) -> Result<Self> {
        let body_bytes = body.collect().await.map_err(|_| Error::NotFound)?.to_bytes();
        let data = serde_json::from_slice(&body_bytes).map_err(|_| Error::NotFound)?;
        Ok(Json(data))
    }
} 