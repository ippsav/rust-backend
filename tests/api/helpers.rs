use axum::{async_trait, Router};
use hyper::{Body, Response};
use lib::configuration::AppConfig;
use serde_json::Value;

use std::net::TcpListener;

#[derive(Debug)]
pub struct TestApp {
    pub config: AppConfig,
}

impl TestApp {
    pub fn build() -> Self {
        env_logger::init();
        // Parse Config
        let config = AppConfig::build("TEST".into()).unwrap();

        Self { config }
    }

    pub fn start_server(&mut self) {
        // Create listener
        let address = self.config.app_settings.address();
        let listener = TcpListener::bind(address).expect("could not bind listener");
        // Setting the used port in the config
        self.config.app_settings.port = listener.local_addr().unwrap().port();
        // Create server
        let router = lib::router::setup_router();
        // Spawn server
        spawn_server(listener, router);
    }

    pub fn get_http_uri(&self, path: &'static str) -> String {
        format!(
            "http://{}:{}{}",
            &self.config.app_settings.host, self.config.app_settings.port, path
        )
    }
}

fn spawn_server(listener: TcpListener, router: Router) {
    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .expect("could not bind the tcp listener")
            .serve(router.into_make_service())
            .await
            .expect("could not start server")
    });
}

#[async_trait]
pub trait Json {
    async fn json_from_body(self) -> Value;
}

#[async_trait]
impl Json for Response<Body> {
    async fn json_from_body(self) -> Value {
        let body = hyper::body::to_bytes(self.into_body())
            .await
            .expect("could not convert body to bytes");
        let value: Value = serde_json::from_slice(&body).expect("could not deserialize body");
        value
    }
}
