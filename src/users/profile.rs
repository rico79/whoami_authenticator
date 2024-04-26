use askama_axum::{IntoResponse, Template};
use axum::{extract::State, Form};
use serde::Deserialize;

use crate::{auth::IdTokenClaims, general::PageMessage, AppState};

use super::{confirm::EmailConfirmation, User};

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "users/profile.html")]
pub struct PageTemplate {
    claims: Option<IdTokenClaims>,
    user: Option<User>,
    confirm_send_url: String,
    password_message: PageMessage,
}

impl PageTemplate {
    /// Prepare page from info
    pub async fn from(
        state: &AppState,
        claims: IdTokenClaims,
        returned_user: Option<User>,
        password_message: PageMessage,
    ) -> Self {
        // Get user
        let user = match returned_user {
            None => User::select_from_id(&state, &claims.sub).await.ok(),
            Some(user) => Some(user),
        };

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
            password_message,
        }
    }
}

/// Get handler
/// Returns the page using the dedicated HTML template
pub async fn get(claims: IdTokenClaims, State(state): State<AppState>) -> impl IntoResponse {
    PageTemplate::from(&state, claims, None, PageMessage::empty()).await
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

    PageTemplate::from(&state, claims, user, PageMessage::empty()).await
}

/// Form
/// Data expected to update
#[derive(Deserialize)]
pub struct PasswordForm {
    password: String,
    confirm_password: String,
}

/// Profile update handler
pub async fn update_password(
    claims: IdTokenClaims,
    State(state): State<AppState>,
    Form(form): Form<PasswordForm>,
) -> impl IntoResponse {
    // Update password and get user
    let user =
        User::update_password(&state, &claims.sub, &form.password, &form.confirm_password).await;

    // Check user
    match user {
        Ok(returned_user) => {
            PageTemplate::from(
                &state,
                claims,
                Some(returned_user),
                PageMessage::from_body(
                    "success".to_owned(),
                    "Votre password a bien été modifié".to_owned(),
                ),
            )
            .await
        }

        Err(error) => {
            PageTemplate::from(
                &state,
                claims,
                None,
                PageMessage::from_body(
                    "negative".to_owned(),
                    error.to_string(),
                ),
            )
            .await
        }
    }
}
