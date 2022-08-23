use crate::handler::status_handler;
use axum::{routing::get, Router};
use tower_http::trace::TraceLayer;

pub fn setup_router() -> Router {
    let router = Router::new()
        .route("/status", get(status_handler))
        .layer(TraceLayer::new_for_http());

    router
}
