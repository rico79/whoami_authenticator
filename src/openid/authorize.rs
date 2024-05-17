use askama_axum::IntoResponse;
use axum::{extract::Query, Form};
use serde::Deserialize;

use super::OpenIdConnectError;

#[derive(Debug, Deserialize)]
pub struct AuthenticationRequest {
    scope: Option<String>,
    response_type: Option<String>,
    client_id: Option<String>,
    redirect_uri: Option<String>,
}

pub async fn get_handler(
    Query(query): Query<AuthenticationRequest>,
) -> Result<impl IntoResponse, OpenIdConnectError> {
    authorize_handler(query)
}

pub async fn post_handler(
    Form(form): Form<AuthenticationRequest>,
) -> Result<impl IntoResponse, OpenIdConnectError> {
    authorize_handler(form)
}

pub fn authorize_handler(
    auth_request: AuthenticationRequest,
) -> Result<impl IntoResponse, OpenIdConnectError> {
    Ok(format!("{:?}", auth_request).into_response())
}
