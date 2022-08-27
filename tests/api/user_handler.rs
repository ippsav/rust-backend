use hyper::{Body, Method, Request};
use serde::Deserialize;
use serde_json::json;

use crate::helpers::{ParseJson, TestApp};

#[derive(Deserialize, Debug)]
pub struct ApiResponse {
    pub token: String,
}

#[tokio::test]
async fn register_handler_success_with_token() {
    let mut app = TestApp::build();
    app.start_server().await;

    // Creating client
    let client = hyper::Client::new();

    let user_input = json!({
        "email": "test@email.com",
        "username": "test_username",
        "password": "test_password"
    });

    let req = Request::builder()
        .method(Method::POST)
        .uri(app.get_http_uri("/api/users/register"))
        .header("Content-Type", "application/json")
        .body(Body::from(user_input.to_string()))
        .expect("could not create request");

    let response = client.request(req).await.expect("could not send request");

    app.teardown().await;

    assert!(response.status().is_success());

    // Getting json data
    let value = response.json_from_body().await;

    let api_response: ApiResponse = serde_json::from_value(value).unwrap();

    assert!(!api_response.token.is_empty())
}
