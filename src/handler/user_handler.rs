use axum::{http::status, response::IntoResponse, Extension, Json};
use chrono::Duration;
use jsonwebtoken::{EncodingKey, Header};
use serde_json::json;
use std::sync::Arc;
use thiserror::Error;
use validator::{Validate, ValidationErrors};

use crate::{
    db::user::{create_user, user_exists_by_username_or_email},
    domain::user::{Claims, CreateUser},
    router::State,
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
    #[error("error encoding jwt")]
    JWTEncoding(#[from] jsonwebtoken::errors::Error),
}

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
            ApiError::HashingPassword | ApiError::DbInternalError(_) | ApiError::JWTEncoding(_) => {
                status::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

#[tracing::instrument(err)]
pub async fn register_handler(
    Json(user_input): Json<CreateUser>,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Validating user_input
    user_input.validate()?;
    let state = state.clone();

    // Check if user already exists
    let count =
        user_exists_by_username_or_email(&user_input.username, &user_input.email, &state.db_pool)
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

    let user = create_user(user_input, &state.db_pool).await?;

    let now = chrono::Utc::now();

    let claims = Claims {
        sub: user.id.to_string(),
        iat: now.clone(),
        exp: now + Duration::hours(4),
        user,
    };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&state.jwt_secret.as_bytes()),
    )?;

    let value = json!({ "token": token });

    Ok(Json(value))
}
