use assert_json_diff::assert_json_include;
use hyper::{Body, Method, Request};
use lib::domain::user::User;
use serde::Deserialize;
use serde_json::json;

use crate::helpers::{app::TestApp, ParseJson};

#[derive(Deserialize, Debug)]
pub struct ApiResponse {
    pub token: String,
    pub user: User,
}

#[tokio::test]
async fn register_handler_success_with_token() {
    let mut app = TestApp::build();
    app.start_server().await;

    // Creating client
    let client = hyper::Client::new();

    let expected_username = "test_username";
    let expected_email = "test@email.com";

    let user_input = json!({
        "email": expected_email,
        "username": expected_username,
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

    let api_response = response.json_from_body().await;

    let token = api_response["token"].to_string();

    assert!(!token.is_empty());

    assert_json_include!(
        actual: api_response,
        expected: json!({
            "user": {
                "username": expected_username,
                "email":expected_email,
            }
        })
    )
}

#[tokio::test]
async fn register_handler_with_validation_errors() {
    let mut app = TestApp::build();
    app.start_server().await;

    // Creating client
    let client = hyper::Client::new();

    let bad_username = "us";
    let bad_email = "testemail.com";

    let user_input = json!({
        "email":  bad_email,
        "username": bad_username,
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

    assert!(response.status().is_client_error());

    // Getting json data

    let api_response = response.json_from_body().await;

    assert_json_include!(
        actual: api_response,
        expected: json!({
        "message": "error validating fields",
        "error": {
            "fields": {
                "email": "invalid email",
                "username": "invalid length"
            }
        }})
    );
}

#[tokio::test]
async fn register_handler_already_registered() {
    let mut app = TestApp::build();
    app.start_server().await;

    // Creating client
    let client = hyper::Client::new();

    let taken_username = "username";
    let taken_email = "test@email.com";

    let user_input = json!({
        "email":  taken_email,
        "username": taken_username,
        "password": "test_password"
    });

    let mut req = Request::builder()
        .method(Method::POST)
        .uri(app.get_http_uri("/api/users/register"))
        .header("Content-Type", "application/json")
        .body(Body::from(user_input.to_string()))
        .expect("could not create request");

    client.request(req).await.expect("could not send request");

    req = Request::builder()
        .method(Method::POST)
        .uri(app.get_http_uri("/api/users/register"))
        .header("Content-Type", "application/json")
        .body(Body::from(user_input.to_string()))
        .expect("could not create request");

    let response = client.request(req).await.expect("could not send request");

    app.teardown().await;

    assert!(response.status().is_client_error());

    // Getting json data

    let api_response = response.json_from_body().await;

    assert_json_include!(
        actual: api_response,
        expected: json!({
        "message": "user already registered",
        })
    );
}
