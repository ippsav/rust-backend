use axum::async_trait;
use hyper::{Body, Response};
use serde_json::Value;

#[async_trait]
pub trait ParseJson {
    async fn json_from_body(self) -> Value;
}

#[async_trait]
impl ParseJson for Response<Body> {
    async fn json_from_body(self) -> Value {
        let body = hyper::body::to_bytes(self.into_body())
            .await
            .expect("could not convert body to bytes");
        let value = serde_json::from_slice(&body).expect("could not deserialize body");

        value
    }
}
