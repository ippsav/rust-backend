use lib::configuration;
use lib::{router::setup_router, server::make_server};
use sqlx::PgPool;
use std::io;
use std::net::TcpListener;
use thiserror::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    dotenv::dotenv().ok();
    let env = dotenv::var("ENVIRONMENT").expect("could not find var ENVIRONMENT");
    // Parse Config
    let config = configuration::AppConfig::build(env)?;

    // Setup Database connection pool
    let db_uri = config.database_settings.connection_string_with_db_name();
    let db_pool = PgPool::connect(&db_uri).await.unwrap();

    // Setup listener
    let address = config.app_settings.address();
    let listener = TcpListener::bind(address)?;

    // Setup router
    let router = setup_router(db_pool);

    make_server(listener, router).await?;
    Ok(())
}

#[derive(Error, Debug)]
enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Server(#[from] hyper::Error),
    #[error(transparent)]
    Config(#[from] config::ConfigError),
}
