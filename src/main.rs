pub mod util;
use axum::extract::Path;
use axum::http::{HeaderMap, HeaderValue};
use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
  routing::get,
  Json, Router,
};
use serde_json::json;
use std::net::SocketAddr;
use url::Url;
use util::get_songs_as_rss;

#[tokio::main]
async fn main() {
  let env = tracing_subscriber::EnvFilter::from("info");
  tracing_subscriber::fmt().with_env_filter(env).init();

  let app = Router::new().route("/:title/:url", get(trans));

  let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
  tracing::info!("listening on {}", addr);
  axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .await
    .unwrap();
}

// basic handler that responds with a static string
async fn trans(
  Path((title, url_base64)): Path<(String, String)>,
) -> Result<(HeaderMap, String), AppError> {
  let url = base64_url::decode(&url_base64)?;
  let url = String::from_utf8_lossy(&url);
  let channel = rss::ChannelBuilder::default()
    .title(title)
    .link(url.clone())
    .items(get_songs_as_rss(Url::parse(&url)?).await?)
    .build();
  let mut headers = HeaderMap::new();
  headers.insert(
    "Content-Type",
    HeaderValue::from_static("application/rss+xml"),
  );
  Ok((headers, channel.to_string()))
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
  #[error("invalid base64 url")]
  InvalidBase64(#[from] base64::DecodeError),
  #[error("failed to request http")]
  HttpClientError(#[from] reqwest::Error),
  #[error("failed to parse url")]
  UrlParseError(#[from] url::ParseError),
  #[error("unknown error")]
  Unknown,
}

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    let (status, error_message) = match &self {
      AppError::InvalidBase64(_) => (StatusCode::BAD_REQUEST, format!("{}", self).to_string()),
      AppError::Unknown => (StatusCode::BAD_REQUEST, format!("{}", self).to_string()),
      AppError::HttpClientError(_) => (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("{}", self).to_string(),
      ),
      AppError::UrlParseError(_) => (StatusCode::BAD_REQUEST, format!("{}", self).to_string()),
    };

    let body = Json(json!({
        "error": error_message,
    }));

    (status, body).into_response()
  }
}
