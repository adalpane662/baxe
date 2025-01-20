use std::fmt::Display;
use axum::http::StatusCode;

pub trait BackendError: std::error::Error {
    fn to_status_code(&self) -> StatusCode;
    fn to_error_tag(&self) -> impl Display;
    fn to_error_code(&self) -> u16;
}
