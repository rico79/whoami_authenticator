use askama_axum::{IntoResponse, Template};
use axum::extract::State;

use crate::{
    apps::{app_list::AppListTemplate, App},
    auth::IdTokenClaims,
    AppState,
};

use super::navbar::NavBarTemplate;

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "general/welcome.html")]
pub struct PageTemplate {
    navbar: NavBarTemplate,
    own_apps: AppListTemplate,
}

/// Get handler
/// Returns the page using the dedicated HTML template
pub async fn get(claims: IdTokenClaims, State(state): State<AppState>) -> impl IntoResponse {
    PageTemplate {
        navbar: NavBarTemplate {
            claims: Some(claims.clone()),
        },
        own_apps: AppListTemplate {
            apps: App::select_own_apps(&state, &claims).await.unwrap(),
        },
    }
}
