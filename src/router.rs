use crate::handler::status_handler;
use axum::{routing::get, Router};

pub fn setup_router() -> Router {
    let router = Router::new().route("/status", get(status_handler));

    router
}
