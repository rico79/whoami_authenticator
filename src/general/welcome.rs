use askama_axum::{IntoResponse, Template};
use axum::extract::State;

use crate::{
    apps::{app_list::AppListTemplate, App},
    auth::IdTokenClaims,
    AppState,
};

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "general/welcome.html")]
pub struct PageTemplate {
    claims: Option<IdTokenClaims>,
    own_apps: AppListTemplate,
}

/// Get handler
/// Returns the page using the dedicated HTML template
pub async fn get(claims: IdTokenClaims, State(state): State<AppState>) -> impl IntoResponse {
    PageTemplate {
        claims: Some(claims.clone()),
        own_apps: AppListTemplate {
            apps: App::select_own_apps(&state, &claims.sub).await.unwrap(),
        },
    }
}
