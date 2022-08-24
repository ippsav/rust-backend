use crate::handler::status_handler;
use axum::{routing::get, Extension, Router};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub struct State {
    pub db_pool: PgPool,
}

pub fn setup_router(db_pool: PgPool) -> Router {
    let state = Arc::new(State { db_pool });
    Router::new()
        .route("/status", get(status_handler))
        .layer(Extension(state))
        .layer(TraceLayer::new_for_http())
}
