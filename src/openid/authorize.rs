use askama_axum::IntoResponse;
use axum::{
    extract::{Query, State},
    Form,
};
use http::Uri;
use serde::Deserialize;

use crate::{
    apps::App,
    auth::{
        signin::{self, SigninPage},
        IdSession,
    },
    AppState,
};

use super::OpenIdConnectError;

#[derive(Debug, Deserialize)]
pub struct AuthenticationRequest {
    scope: Option<String>,
    response_type: Option<String>,
    client_id: Option<String>,
    redirect_uri: Option<String>,
}

pub async fn get_handler(
    id_session: Option<IdSession>,
    request_uri: Uri,
    State(state): State<AppState>,
    Query(query): Query<AuthenticationRequest>,
) -> Result<impl IntoResponse, OpenIdConnectError> {
    authorize_handler(id_session, state, query, request_uri).await
}

pub async fn post_handler(
    id_session: Option<IdSession>,
    request_uri: Uri,
    State(state): State<AppState>,
    Form(form): Form<AuthenticationRequest>,
) -> Result<impl IntoResponse, OpenIdConnectError> {
    authorize_handler(id_session, state, form, request_uri).await
}

pub async fn authorize_handler(
    id_session: Option<IdSession>,
    state: AppState,
    auth_request: AuthenticationRequest,
    request_uri: Uri,
) -> Result<impl IntoResponse, OpenIdConnectError> {
    let redirect_uri = validate_redirect_uri(auth_request.redirect_uri)?;

    let response_type = validate_response_type(auth_request.response_type, redirect_uri.clone())?;

    let scope = validate_scope(auth_request.scope, redirect_uri.clone())?;

    let app_to_connect_to =
        validate_client_id(&state, auth_request.client_id, redirect_uri.clone()).await?;

    let already_connected = id_session.is_some();

    if already_connected {
        Ok(app_to_connect_to
            .redirect_to_endpoint(Some(format!(
                "{}?result=ok",
                redirect_uri.path().to_string()
            )))
            .into_response())
    } else {
        Ok(SigninPage::for_app_from_query(
            app_to_connect_to.clone(),
            signin::QueryParams {
                mail: None,
                app_id: Some(app_to_connect_to.id),
                requested_endpoint: Some(request_uri.to_string()),
            },
        )
        .into_response())
    }
}

fn validate_redirect_uri(redirect_uri: Option<String>) -> Result<Uri, OpenIdConnectError> {
    match redirect_uri {
        Some(redirect_uri) => redirect_uri
            .parse::<Uri>()
            .map_err(|_| OpenIdConnectError::InvalidRequest(None)),

        None => Err(OpenIdConnectError::InvalidRequest(None)),
    }
}

fn validate_response_type(
    response_type: Option<String>,
    redirect_uri: Uri,
) -> Result<String, OpenIdConnectError> {
    match response_type {
        Some(response_type) => {
            if response_type.contains("code") {
                Ok(response_type)
            } else {
                Err(OpenIdConnectError::UnsupportedResponseType(redirect_uri))
            }
        }

        None => Err(OpenIdConnectError::UnsupportedResponseType(redirect_uri)),
    }
}

fn validate_scope(scope: Option<String>, redirect_uri: Uri) -> Result<String, OpenIdConnectError> {
    match scope {
        Some(scope) => {
            if scope.contains("openid") {
                Ok(scope)
            } else {
                Err(OpenIdConnectError::InvalidScope(redirect_uri))
            }
        }

        None => Err(OpenIdConnectError::InvalidScope(redirect_uri)),
    }
}

async fn validate_client_id(
    state: &AppState,
    client_id: Option<String>,
    redirect_uri: Uri,
) -> Result<App, OpenIdConnectError> {
    match client_id {
        Some(client_id) => {
            let app_id: i32 = client_id
                .parse()
                .map_err(|_| OpenIdConnectError::UnauthorizedClient(redirect_uri.clone()))?;

            let app = App::select_from_app_id(state, app_id)
                .await
                .map_err(|_| OpenIdConnectError::UnauthorizedClient(redirect_uri.clone()))?;

            if app.redirect_url() != redirect_uri.to_string() {
                Err(OpenIdConnectError::UnauthorizedClient(redirect_uri.clone()))
            } else {
                Ok(app)
            }
        }

        None => Err(OpenIdConnectError::UnauthorizedClient(redirect_uri)),
    }
}
