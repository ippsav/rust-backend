use lib::configuration;
use lib::{router::setup_router, server::make_server};
use std::io;
use std::net::TcpListener;
use thiserror::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();
    let env = dotenv::var("ENVIRONMENT").expect("could not find var ENVIRONMENT");
    // Parse Config
    let config = configuration::AppConfig::build(env)?;

    // Setup listener
    let address = config.app_settings.address();
    let listener = TcpListener::bind(address)?;

    // Setup router
    let router = setup_router();

    make_server(listener, router).await?;
    Ok(())
}

#[derive(Error, Debug)]
enum Error {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    ServerError(#[from] hyper::Error),
    #[error(transparent)]
    ConfigError(#[from] config::ConfigError),
}
