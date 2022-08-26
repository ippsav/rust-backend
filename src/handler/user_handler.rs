use axum::{http::status, response::IntoResponse, Extension, Json};
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;
use thiserror::Error;
use validator::{Validate, ValidationErrors};

use crate::{
    db::user::{create_user, user_exists_by_username_or_email},
    domain::user::{CreateUser, User},
    utils::hasher::hash_password,
};

#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    BadClientData(#[from] ValidationErrors),
    #[error("user already registered")]
    UserAlreadyRegistered,
    #[error("could not hash password")]
    HashingPassword,
    #[error(transparent)]
    DbInternalError(#[from] sqlx::Error),
}

#[derive(Serialize)]
pub struct ApiResponse {}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::UserAlreadyRegistered => {
                (status::StatusCode::CONFLICT, "user already registered\n").into_response()
            }
            ApiError::BadClientData(err) => {
                let errors = err
                    .field_errors()
                    .into_keys()
                    .map(String::from)
                    .collect::<Vec<_>>()
                    .join(",");
                (
                    status::StatusCode::BAD_REQUEST,
                    format!("bad client data: {}\n", errors),
                )
                    .into_response()
            }
            ApiError::HashingPassword | ApiError::DbInternalError(_) => {
                status::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

#[tracing::instrument(err)]
pub async fn register_handler(
    Json(user_input): Json<CreateUser>,
    Extension(db_pool): Extension<Arc<PgPool>>,
) -> Result<Json<User>, ApiError> {
    // Validating user_input
    user_input.validate()?;

    // Check if user already exists
    let count = user_exists_by_username_or_email(&user_input.username, &user_input.email, &db_pool)
        .await?
        .map_or(0, |x| x);

    if count != 0 {
        return Err(ApiError::UserAlreadyRegistered);
    }

    // Hash password
    let hashed_password =
        hash_password(user_input.password.as_bytes()).map_err(|_| ApiError::HashingPassword)?;

    // Inserting User
    let user_input = CreateUser {
        password: hashed_password,
        ..user_input
    };

    let db_pool = db_pool.clone();

    let user = create_user(user_input, &db_pool).await?;

    Ok(Json(user))
}
