use axum::Router;
use lib::configuration::{AppConfig, DatabaseSettings};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;

use std::net::TcpListener;

#[derive(Debug)]
pub struct TestApp {
    pub config: AppConfig,
}

impl TestApp {
    pub fn build() -> Self {
        let config = AppConfig::build("TEST".into()).unwrap();

        Self { config }
    }

    pub async fn start_server(&mut self) {
        // Create Database connection pool
        let db_pool = Self::setup_db(&mut self.config.database_settings).await;

        // Create listener
        let address = self.config.app_settings.address();
        let listener = TcpListener::bind(address).expect("could not bind listener");

        // Setting the used port in the config
        self.config.app_settings.port = listener.local_addr().unwrap().port();

        // Create server
        let router =
            lib::router::setup_router(db_pool, self.config.app_settings.jwt_secret.clone());

        // Spawn server
        spawn_server(listener, router);
    }

    pub async fn teardown(self) {
        let mut conn = PgConnection::connect(&self.config.database_settings.connection_string())
            .await
            .expect("could not connect to db");

        // Disconnect any existing connections to the DB
        let sql = format!(
            r#"SELECT pg_terminate_backend(pg_stat_activity.pid)
FROM pg_stat_activity
WHERE pg_stat_activity.datname = '{db}'
AND pid <> pg_backend_pid();"#,
            db = &self.config.database_settings.db_name,
        );
        sqlx::query(&sql).execute(&mut conn).await.unwrap();

        // Drop db
        let sql = format!(
            r#"DROP DATABASE "{}";"#,
            &self.config.database_settings.db_name
        );
        conn.execute(sql.as_str())
            .await
            .expect("could not drop database");
        conn.close().await.expect("could not close connection");
    }

    async fn setup_db(db_settings: &mut DatabaseSettings) -> PgPool {
        // Creating Connection to database without db_name
        let mut db_uri = db_settings.connection_string();
        let mut conn = PgConnection::connect(&db_uri)
            .await
            .expect("could not connect to a db");

        db_settings.db_name = Uuid::new_v4().to_string();

        let sql = format!(r#"CREATE DATABASE "{}";"#, &db_settings.db_name);

        // Creating database
        conn.execute(sql.as_str())
            .await
            .expect("could not create database");

        // Creating database connection pool
        db_uri = db_settings.connection_string_with_db_name();
        let db_pool = PgPool::connect(&db_uri)
            .await
            .expect("could not connect to database");

        // Running migrations
        sqlx::migrate!("./migrations")
            .run(&db_pool)
            .await
            .expect("could not run database migrations");

        db_pool
    }

    pub fn get_http_uri(&self, path: &'static str) -> String {
        format!(
            "http://{}:{}{}",
            &self.config.app_settings.host, self.config.app_settings.port, path
        )
    }
}

fn spawn_server(listener: TcpListener, router: Router) {
    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .expect("could not bind the tcp listener")
            .serve(router.into_make_service())
            .await
            .expect("could not start server")
    });
}
