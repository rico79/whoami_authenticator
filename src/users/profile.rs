use askama_axum::{IntoResponse, Template};
use axum::{extract::State, Form};
use serde::Deserialize;

use crate::{auth::IdTokenClaims, AppState};

use super::User;

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "users/profile.html")]
pub struct PageTemplate {
    claims: Option<IdTokenClaims>,
    user: Option<User>,
}

/// Get handler
/// Returns the page using the dedicated HTML template
pub async fn get(claims: IdTokenClaims, State(state): State<AppState>) -> impl IntoResponse {
    PageTemplate {
        claims: Some(claims.clone()),
        user: User::select_from_id(&state, &claims.sub).await.ok(),
    }
}

/// Profile form
/// Data expected to update
#[derive(Deserialize)]
pub struct ProfileForm {
    name: String,
    email: String,
}

/// Profile update handler
pub async fn update_profile(
    claims: IdTokenClaims,
    State(state): State<AppState>,
    Form(form): Form<ProfileForm>,
) -> impl IntoResponse {
    PageTemplate {
        claims: Some(claims.clone()),
        user: User::update_profile(&state, &claims.sub, &form.name, &form.email).await.ok(),
    }
}
