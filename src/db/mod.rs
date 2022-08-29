pub mod user;

#[cfg(test)]
mod test_utils {
    use sqlx::{Connection, Executor, PgConnection, PgPool};
    use uuid::Uuid;

    use crate::configuration::AppConfig;

    pub async fn configure_database() -> (AppConfig, PgPool) {
        let mut config = AppConfig::build("TEST".into()).unwrap();

        // Creating Connection to database without db_name
        let mut db_uri = config.database_settings.connection_string();
        let mut conn = PgConnection::connect(&db_uri)
            .await
            .expect("could not connect to a db");

        config.database_settings.db_name = Uuid::new_v4().to_string();

        let sql = format!(
            r#"CREATE DATABASE "{}";"#,
            &config.database_settings.db_name
        );

        // Creating database
        conn.execute(sql.as_str())
            .await
            .expect("could not create database");

        // Creating database connection pool
        db_uri = config.database_settings.connection_string_with_db_name();
        let db_pool = PgPool::connect(&db_uri)
            .await
            .expect("could not connect to database");

        // Running migrations
        sqlx::migrate!("./migrations")
            .run(&db_pool)
            .await
            .expect("could not run database migrations");

        (config, db_pool)
    }

    pub async fn drop_db(config: AppConfig, db_pool: PgPool) {
        db_pool.close().await;
        let mut conn = PgConnection::connect(&config.database_settings.connection_string())
            .await
            .expect("could not connect to db");

        // Disconnect any existing connections to the DB
        let sql = format!(
            r#"SELECT pg_terminate_backend(pg_stat_activity.pid)
FROM pg_stat_activity
WHERE pg_stat_activity.datname = '{db}'
AND pid <> pg_backend_pid();"#,
            db = &config.database_settings.db_name,
        );
        sqlx::query(&sql).execute(&mut conn).await.unwrap();

        // Drop db
        let sql = format!(r#"DROP DATABASE "{}";"#, &config.database_settings.db_name);
        conn.execute(sql.as_str())
            .await
            .expect("could not drop database");
        conn.close().await.expect("could not close connection");
    }
}
