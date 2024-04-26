use askama_axum::{IntoResponse, Template};
use axum::{extract::State, Form};
use serde::Deserialize;

use crate::{auth::IdTokenClaims, AppState};

use super::{confirm::EmailConfirmation, User};

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "users/profile.html")]
pub struct PageTemplate {
    claims: Option<IdTokenClaims>,
    user: Option<User>,
    confirm_send_url: String,
}

/// Get handler
/// Returns the page using the dedicated HTML template
pub async fn get(claims: IdTokenClaims, State(state): State<AppState>) -> impl IntoResponse {
    // Get user
    let user = User::select_from_id(&state, &claims.sub).await.ok();

    // Prepare confirmation sending url
    let confirm_send_url = match &user {
        Some(user) => {
            EmailConfirmation::from(&state, user.clone(), state.authenticator_app.clone())
                .send_url()
        }
        None => "".to_owned(),
    };

    PageTemplate {
        claims: Some(claims.clone()),
        user: user,
        confirm_send_url,
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
    // Update profile and get user
    let user = User::update_profile(&state, &claims.sub, &form.name, &form.email)
        .await
        .ok();

    // Prepare confirmation sending url
    let confirm_send_url = match &user {
        Some(user) => {
            EmailConfirmation::from(&state, user.clone(), state.authenticator_app.clone())
                .send_url()
        }
        None => "".to_owned(),
    };

    PageTemplate {
        claims: Some(claims.clone()),
        user: user,
        confirm_send_url,
    }
}
