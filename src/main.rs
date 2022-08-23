use lib::{router::setup_router, server::make_server};
use std::io;
use std::net::TcpListener;
use thiserror::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Setup listener
    let listener = TcpListener::bind("127.0.0.1:3000")?;

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
}
