use askama_axum::{IntoResponse, Template};
use axum::{extract::State, Form};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
    auth::{create_session_into_response, IdTokenClaims},
    general::{go_back::GoBackTemplate, message::MessageTemplate, navbar::NavBarTemplate},
    AppState,
};

use super::{confirm::MailConfirmation, User};

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "users/profile.html")]
pub struct PageTemplate {
    navbar: NavBarTemplate,
    go_back: GoBackTemplate,
    user: Option<User>,
    confirm_send_url: String,
    profile_message: MessageTemplate,
    password_message: MessageTemplate,
}

impl PageTemplate {
    /// Prepare page from info
    pub async fn from(
        state: &AppState,
        claims: IdTokenClaims,
        returned_user: Option<User>,
        profile_message: MessageTemplate,
        password_message: MessageTemplate,
    ) -> Self {
        // Get user
        let user = match returned_user {
            None => User::select_from_id(&state, claims.user_id()).await.ok(),
            Some(user) => Some(user),
        };

        // Prepare confirmation sending url
        let confirm_send_url = match &user {
            Some(user) => {
                MailConfirmation::from(&state, user.clone(), state.authenticator_app.clone())
                    .send_url()
            }
            None => "".to_owned(),
        };

        PageTemplate {
            navbar: NavBarTemplate {
                claims: Some(claims),
            },
            go_back: GoBackTemplate {
                back_url: "/home".to_owned(),
            },
            user: user,
            confirm_send_url,
            profile_message,
            password_message,
        }
    }
}

/// Get handler
/// Returns the page using the dedicated HTML template
pub async fn get(claims: IdTokenClaims, State(state): State<AppState>) -> impl IntoResponse {
    PageTemplate::from(
        &state,
        claims,
        None,
        MessageTemplate::empty(),
        MessageTemplate::empty(),
    )
    .await
}

/// Profile form
/// Data expected to update
#[derive(Deserialize)]
pub struct ProfileForm {
    name: String,
    mail: String,
    birthday: String,
    avatar_url: String,
}

/// Profile update handler
pub async fn update_profile(
    cookies: CookieJar,
    claims: IdTokenClaims,
    State(state): State<AppState>,
    Form(form): Form<ProfileForm>,
) -> impl IntoResponse {
    // Update profile and get user
    let user = User::update_profile(
        &state,
        &claims.user_id(),
        &form.name,
        &form.birthday,
        &form.avatar_url,
        &form.mail,
    )
    .await;

    match user {
        Ok(updated_user) => {
            // Renew the token if profile updated
            let claims = IdTokenClaims::new(
                &state,
                updated_user.id,
                updated_user.name.clone(),
                updated_user.mail.clone(),
                state.authenticator_app.jwt_seconds_to_expire.clone(),
            );

            // Check token
            if let Ok(jwt) = claims.encode(state.authenticator_app.jwt_secret.clone()) {
                create_session_into_response(
                    cookies,
                    jwt,
                    PageTemplate::from(
                        &state,
                        claims,
                        Some(updated_user),
                        MessageTemplate::empty(),
                        MessageTemplate::empty(),
                    )
                    .await,
                )
                .into_response()
            } else {
                PageTemplate::from(
                    &state,
                    claims,
                    None,
                    MessageTemplate::empty(),
                    MessageTemplate::empty(),
                )
                .await
                .into_response()
            }
        }
        Err(error) => PageTemplate::from(
            &state,
            claims,
            None,
            MessageTemplate::from_body("negative".to_owned(), error.to_string(), true),
            MessageTemplate::empty(),
        )
        .await
        .into_response(),
    }
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
    let user = User::update_password(
        &state,
        &claims.user_id(),
        &form.password,
        &form.confirm_password,
    )
    .await;

    // Check user
    match user {
        Ok(_) => MessageTemplate::from_body(
            "success".to_owned(),
            "Votre password a bien été modifié".to_owned(),
            true,
        ),

        Err(error) => MessageTemplate::from_body("negative".to_owned(), error.to_string(), true),
    }
}
