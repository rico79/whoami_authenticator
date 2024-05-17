use std::fmt;

use askama_axum::IntoResponse;
use http::StatusCode;

pub mod authorize;

#[derive(Debug)]
pub enum OpenIdConnectError {}

impl fmt::Display for OpenIdConnectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl IntoResponse for OpenIdConnectError {
    fn into_response(self) -> askama_axum::Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}
