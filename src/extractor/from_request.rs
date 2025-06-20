use crate::{
    http_request::{Body, Request, RequestParts},
    Result,
};
use async_trait::async_trait;

#[async_trait]
pub trait FromRequest: Sized {
    async fn from_request(parts: &mut RequestParts, body: Body) -> Result<Self>;
} 