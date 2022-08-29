use lib::configuration;
use lib::{router::setup_router, server::make_server};
use sqlx::PgPool;
use std::io;
use std::net::TcpListener;
use thiserror::Error;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // a builder for `FmtSubscriber`.
    let subscriber = FmtSubscriber::builder()
        // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
        // will be written to stdout.
        .with_max_level(Level::TRACE)
        // completes the builder.
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

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
    let router = setup_router(db_pool, config.app_settings.jwt_secret);

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
