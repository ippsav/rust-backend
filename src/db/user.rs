use crate::domain::user::{CreateUser, User};
use sqlx::PgPool;
use uuid::Uuid;

#[tracing::instrument(err)]
pub async fn create_user(user_input: CreateUser, db_pool: &PgPool) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as!(
        User,
        r#"
    INSERT INTO users(id, username, email,password_hash) values($1,$2,$3,$4) RETURNING *;
    "#,
        Uuid::new_v4(),
        user_input.username,
        user_input.email,
        user_input.password
    )
    .fetch_one(db_pool)
    .await?;

    Ok(user)
}

pub async fn find_user_by_username(
    username: &str,
    db_pool: &PgPool,
) -> Result<Option<User>, sqlx::Error> {
    let user = sqlx::query_as!(User, r#"select * from users where username = $1"#, username)
        .fetch_optional(db_pool)
        .await?;

    Ok(user)
}

#[tracing::instrument(err)]
pub async fn user_exists_by_username_or_email(
    username: &str,
    email: &str,
    db_pool: &PgPool,
) -> Result<Option<i64>, sqlx::Error> {
    let row = sqlx::query!(
        r#"select count(*) from users where username = $1 or email = $2"#,
        username,
        email
    )
    .fetch_one(db_pool)
    .await?;

    Ok(row.count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_utils;

    #[tokio::test]
    async fn create_user_with_success() {
        // Init database
        let (config, db_pool) = test_utils::configure_database().await;
        let username = "username";
        let email = "email@gmail.com";

        // Creating user input
        let user_input = CreateUser {
            username: username.into(),
            email: email.into(),
            password: "password".into(),
        };

        let user = create_user(user_input, &db_pool).await.unwrap();

        // Dropping database
        test_utils::drop_db(config, db_pool).await;

        assert_eq!(user.email, email);
        assert_eq!(user.username, username);
    }

    #[tokio::test]
    async fn find_user_with_success() {
        // Init database
        let (config, db_pool) = test_utils::configure_database().await;
        let username = "username";
        let email = "email@gmail.com";

        // Creating user input
        let user_input = CreateUser {
            username: username.into(),
            email: email.into(),
            password: "password".into(),
        };

        let created_user = create_user(user_input, &db_pool).await.unwrap();

        // Checking inserted user
        let user = find_user_by_username(username, &db_pool)
            .await
            .unwrap()
            .expect("user not found");

        // Dropping database
        test_utils::drop_db(config, db_pool).await;

        assert_eq!(created_user, user);
    }

    #[tokio::test]
    async fn find_user_none() {
        // Init database
        let (config, db_pool) = test_utils::configure_database().await;
        let username = "username";

        // Checking inserted user
        let user = find_user_by_username(username, &db_pool).await.unwrap();

        // Dropping database
        test_utils::drop_db(config, db_pool).await;

        assert!(user.is_none());
    }
}
