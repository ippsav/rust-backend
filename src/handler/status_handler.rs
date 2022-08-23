use axum::response::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct ServerStatus {
    status: &'static str,
}

pub async fn status_handler() -> Json<ServerStatus> {
    Json(ServerStatus { status: "OK" })
}
