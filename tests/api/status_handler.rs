use hyper::{Body, Method, Request};
use serde_json::json;

use crate::helpers::{ParseJson, TestApp};

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
    let value = response.json_from_body().await;

    let expected = json!({
        "status": "OK"
    });

    assert_eq!(expected, value);
}
