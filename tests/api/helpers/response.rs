use axum::async_trait;
use hyper::{Body, Response};
use serde::Deserialize;

#[async_trait]
pub trait ParseJson<T>
where
    for<'de> T: Deserialize<'de>,
{
    async fn json_from_body(self) -> T;
}

#[async_trait]
impl<T> ParseJson<T> for Response<Body>
where
    for<'de> T: Deserialize<'de>,
{
    async fn json_from_body(self) -> T {
        let body = hyper::body::to_bytes(self.into_body())
            .await
            .expect("could not convert body to bytes");
        let value: T = serde_json::from_slice(&body).expect("could not deserialize body");
        value
    }
}
