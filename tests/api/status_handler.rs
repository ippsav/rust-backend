use hyper::{Body, Method, Request};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::helpers::{app::TestApp, ParseJson};
use assert_json_diff::assert_json_include;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct StatusResponse {
    pub status: String,
}

#[tokio::test]
async fn status_handler() {
    let mut app = TestApp::build();
    app.start_server().await;

    // Creating client
    let client = hyper::Client::new();

    let response = client
        .request(
            Request::builder()
                .method(Method::GET)
                .uri(app.get_http_uri("/status"))
                .body(Body::empty())
                .expect("could not create request"),
        )
        .await
        .expect("could not send request");

    app.teardown().await;
    assert!(response.status().is_success());

    // Getting json data
    let value: Value = response.json_from_body().await;

    assert_json_include! {
        actual: value,
        expected: json!({
            "status": "OK"
        })
    }
}
