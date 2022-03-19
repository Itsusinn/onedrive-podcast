use axum::extract::Path;
use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  routing::{get, post},
  Json, Router,
};
use serde_json::json;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  let app = Router::new().route("/:url", get(trans));

  let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
  tracing::debug!("listening on {}", addr);
  axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await
    .unwrap();
}

// basic handler that responds with a static string
async fn trans(Path(url_base64): Path<String>) -> Result<String, AppError> {
  let url = base64_url::decode(&url_base64)?;
  let url = String::from_utf8_lossy(&url);
  Ok(url.into())
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
  #[error("invalid base64 url")]
  InvalidBase64(#[from] base64::DecodeError),
  #[error("unknown error")]
  Unknown,
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    let (status, error_message) = match &self {
      AppError::InvalidBase64(_) => (StatusCode::BAD_REQUEST, format!("{}", self).to_string()),
      AppError::Unknown => (StatusCode::BAD_REQUEST, format!("{}", self).to_string()),
    };

    let body = Json(json!({
        "error": error_message,
    }));

    (status, body).into_response()
  }
}
