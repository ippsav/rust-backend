use axum::{http::status, response::IntoResponse, Extension, Json};
use chrono::Duration;
use jsonwebtoken::{EncodingKey, Header};
use serde::Serialize;
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;
use validator::{Validate, ValidationErrors};

use crate::{
    db::user::{create_user, user_exists_by_username_or_email},
    domain::user::{Claims, CreateUser, User},
    errors::api::ApiErrorResponse,
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
            ApiError::HashingPassword | ApiError::DbInternalError(_) | ApiError::JWTEncoding(_) => {
                status::StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
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
