use hyper::{Body, Request};
use serde_json::{json, Value};

use crate::helpers::TestApp;

#[tokio::test]
async fn status_handler() {
    let app = TestApp::new("127.0.0.1".into(), 3000);
    app.start_server();

    let client = hyper::Client::new();

    let response = client
        .request(
            Request::builder()
                .uri(app.get_http_uri("/status"))
                .body(Body::empty())
                .expect("could not create request"),
        )
        .await
        .expect("could not send request");
    assert!(response.status().is_success());
    let body = hyper::body::to_bytes(response.into_body())
        .await
        .expect("could not convert body to bytes");
    let value: Value = serde_json::from_slice(&body).expect("could not deserialize body");

    let expected = json!({
        "status": "OK"
    });

    assert_eq!(expected, value);
}
