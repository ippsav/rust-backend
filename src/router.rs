use crate::handler::{login_handler, register_handler, status_handler};
use axum::{
    routing::{get, post},
    Extension, Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

#[derive(Debug)]
pub struct State {
    pub db_pool: PgPool,
    pub jwt_secret: String,
}

pub fn setup_router(db_pool: PgPool, jwt_secret: String) -> Router {
    let state = Arc::new(State {
        db_pool,
        jwt_secret,
    });

    let user_routes = Router::new()
        .route("/register", post(register_handler))
        .route("/login", get(login_handler));

    let api_routes = Router::new().nest("/users", user_routes);

    Router::new()
        .route("/status", get(status_handler))
        .nest("/api", api_routes)
        .layer(Extension(state))
        .layer(TraceLayer::new_for_http())
}
