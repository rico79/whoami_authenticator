use askama_axum::IntoResponse;
use axum::response::Redirect;
use http::{StatusCode, Uri};

pub mod authorize;

#[derive(Debug)]
pub enum OpenIdConnectError {
    InvalidRequest(Option<Uri>),
    InvalidScope(Uri),
    UnauthorizedClient(Uri),
    UnsupportedResponseType(Uri),
}

impl IntoResponse for OpenIdConnectError {
    fn into_response(self) -> askama_axum::Response {
        match self {
            OpenIdConnectError::InvalidRequest(Some(redirect_uri)) => Redirect::to(&format!(
                "{}?error=invalid_request",
                redirect_uri.to_string()
            ))
            .into_response(),

            OpenIdConnectError::InvalidRequest(None) => {
                (StatusCode::BAD_REQUEST, "invalid_request").into_response()
            }

            OpenIdConnectError::InvalidScope(redirect_uri) => {
                Redirect::to(&format!("{}?error=invalid_scope", redirect_uri.to_string()))
                    .into_response()
            }

            OpenIdConnectError::UnauthorizedClient(redirect_uri) => Redirect::to(&format!(
                "{}?error=unauthorized_client",
                redirect_uri.to_string()
            ))
            .into_response(),

            OpenIdConnectError::UnsupportedResponseType(redirect_uri) => Redirect::to(&format!(
                "{}?error=unsupported_response_type",
                redirect_uri.to_string()
            ))
            .into_response(),
        }
    }
}
