use axum::{http::status, response::IntoResponse, Extension, Json};
use chrono::Duration;
use jsonwebtoken::{EncodingKey, Header};
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use validator::{Validate, ValidationErrors};

use crate::{
    db::user::{create_user, find_user_by_username, user_exists_by_username_or_email},
    domain::user::{Claims, CreateUser, FindUser, User},
    errors::api::ApiErrorResponse,
    router::State,
    utils::hasher::{hash_password, verify_password},
};

#[derive(Error, Debug)]
pub enum ApiError {
    #[error(transparent)]
    BadClientData(#[from] ValidationErrors),
    #[error("user already registered")]
    UserAlreadyRegistered,
    #[error("user not found")]
    UserNotFound,
    #[error("wrong username or password")]
    BadCredentials,
    #[error("could not hash password")]
    HashError,
    #[error(transparent)]
    DbInternalError(#[from] sqlx::Error),
    #[error("error encoding jwt")]
    JWTEncoding(#[from] jsonwebtoken::errors::Error),
}

#[derive(Serialize, Debug)]
pub struct ResponseErrorObject {
    pub fields: Option<HashMap<String, String>>,
}

impl From<ValidationErrors> for ApiErrorResponse<ResponseErrorObject> {
    fn from(v: ValidationErrors) -> Self {
        let mut hash_map: HashMap<String, String> = HashMap::new();
        v.field_errors().into_iter().for_each(|(k, v)| {
            let msg = format!("invalid {}", v[0].code);

            hash_map.insert(k.into(), msg);
        });

        Self {
            message: "error validating fields".into(),
            error: Some(ResponseErrorObject {
                fields: Some(hash_map),
            }),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::UserNotFound => (
                status::StatusCode::NOT_ACCEPTABLE,
                Json(ApiErrorResponse::<()>::from("user not found")),
            )
                .into_response(),
            ApiError::UserAlreadyRegistered => (
                status::StatusCode::CONFLICT,
                Json(ApiErrorResponse::<()>::from("user already registered")),
            )
                .into_response(),
            ApiError::BadClientData(err) => (
                status::StatusCode::BAD_REQUEST,
                Json(ApiErrorResponse::from(err)),
            )
                .into_response(),
            ApiError::HashError | ApiError::DbInternalError(_) | ApiError::JWTEncoding(_) => {
                status::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            ApiError::BadCredentials => (
                status::StatusCode::NOT_ACCEPTABLE,
                Json(ApiErrorResponse::<()>::from("bad credentials")),
            )
                .into_response(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub token: String,
    pub user: User,
}

#[tracing::instrument(err)]
pub async fn register_handler(
    Json(user_input): Json<CreateUser>,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<ApiResponse>, ApiError> {
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
        hash_password(user_input.password.as_bytes()).map_err(|_| ApiError::HashError)?;

    // Inserting User
    let user_input = CreateUser {
        password: hashed_password,
        ..user_input
    };

    let user = create_user(user_input, &state.db_pool).await?;

    let now = chrono::Utc::now();

    let claims = Claims {
        sub: user.id.to_string(),
        iat: now,
        exp: now + Duration::hours(4),
    };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )?;

    let res = ApiResponse { token, user };

    Ok(Json(res))
}

pub async fn login_handler(
    Json(login_input): Json<FindUser>,
    Extension(state): Extension<Arc<State>>,
) -> Result<Json<ApiResponse>, ApiError> {
    let state = state.clone();

    let user = find_user_by_username(&login_input.username, &state.db_pool)
        .await?
        .ok_or(ApiError::UserNotFound)?;

    let is_match = verify_password(login_input.password.as_bytes(), &user.password_hash)
        .map_err(|_| ApiError::HashError)?;
    if !is_match {
        return Err(ApiError::BadCredentials);
    }

    let now = chrono::Utc::now();

    let claims = Claims {
        sub: user.id.to_string(),
        iat: now,
        exp: now + Duration::hours(4),
    };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )?;

    let res = ApiResponse { token, user };

    Ok(Json(res))
}
