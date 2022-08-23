use std::net::TcpListener;

use axum::Router;
use hyper::Error;

pub async fn make_server(listener: TcpListener, router: Router) -> Result<(), Error> {
    axum::Server::from_tcp(listener)?
        .serve(router.into_make_service())
        .await?;
    Ok(())
}
