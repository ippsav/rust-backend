use crate::handler::{register_handler, status_handler};
use axum::{
    routing::{get, post},
    Extension, Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub fn setup_router(db_pool: PgPool) -> Router {
    let state = Arc::new(db_pool);

    let user_routes = Router::new().route("/register", post(register_handler));

    let api_routes = Router::new().nest("/users", user_routes);

    Router::new()
        .route("/status", get(status_handler))
        .nest("/api", api_routes)
        .layer(Extension(state))
        .layer(TraceLayer::new_for_http())
}
