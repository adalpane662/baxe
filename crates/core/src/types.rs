use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BaxeError {
    #[serde(skip)]
    pub status_code: StatusCode,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub message: String,
    pub code: u16,
    pub error_tag: String,
}

impl std::fmt::Display for BaxeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BaxeError {}

impl IntoResponse for BaxeError {
    fn into_response(self) -> Response {
        (self.status_code, Json(self)).into_response()
    }
}
