use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUser {
    #[validate(length(min = 6, max = 25))]
    pub username: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct FindUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUser {
    #[validate(length(min = 6, max = 25))]
    pub username: String,
    #[validate(length(min = 6))]
    pub old_password: String,
    #[validate(length(min = 6))]
    pub new_password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub sub: String,
    pub iat: DateTime<Utc>,
    pub exp: DateTime<Utc>,
    pub user: User,
}
