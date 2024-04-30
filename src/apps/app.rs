use askama::Template;
use askama_axum::IntoResponse;
use axum::extract::State;

use crate::{auth::IdTokenClaims, AppState};

use super::App;

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "apps/app.html")]
pub struct PageTemplate {
    claims: Option<IdTokenClaims>,
    app: App,
}

/// Get handler
/// Returns the page using the dedicated HTML template
pub async fn get(claims: IdTokenClaims, State(state): State<AppState>) -> impl IntoResponse {
    PageTemplate {
        claims: Some(claims),
        app: state.authenticator_app,
    }
}

/// Post handler
/// Returns the page using the dedicated HTML template
pub async fn post(claims: IdTokenClaims, State(state): State<AppState>) -> impl IntoResponse {
    todo!()
}