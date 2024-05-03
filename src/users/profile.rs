use askama_axum::{IntoResponse, Template};
use axum::{extract::State, Form};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use serde::Deserialize;

use crate::{
    general::{
        go_back::GoBackButton,
        message::{Level, MessageBlock},
        navbar::NavBarBlock,
    },
    utils::jwt::{IdTokenClaims, JsonWebToken},
    AppState,
};

use super::{confirm::ConfirmationMail, User};

#[derive(Template)]
#[template(path = "users/profile_page.html")]
pub struct ProfilePage {
    navbar: NavBarBlock,
    go_back: GoBackButton,
    user: Option<User>,
    confirm_send_url: String,
    profile_message: MessageBlock,
    password_message: MessageBlock,
}

impl ProfilePage {
    pub async fn from(
        state: &AppState,
        claims: IdTokenClaims,
        returned_user: Option<User>,
        profile_message: MessageBlock,
    ) -> Self {
        let user = returned_user.or(User::select_from_id(&state, claims.user_id()).await.ok());

        let confirm_send_url = match &user {
            Some(user) => {
                ConfirmationMail::from(&state, user.clone(), state.authenticator_app.clone())
                    .send_url()
            }
            None => "".to_owned(),
        };

        ProfilePage {
            navbar: NavBarBlock {
                claims: Some(claims),
            },
            go_back: GoBackButton {
                back_url: "/home".to_owned(),
            },
            user: user,
            confirm_send_url,
            profile_message,
            password_message: MessageBlock::empty(),
        }
    }
}

pub async fn get_handler(
    claims: IdTokenClaims,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ProfilePage::from(&state, claims, None, MessageBlock::empty()).await
}

#[derive(Deserialize)]
pub struct ProfileForm {
    name: String,
    mail: String,
    birthday: String,
    avatar_url: String,
}

pub async fn update_profile_handler(
    cookies: CookieJar,
    claims: IdTokenClaims,
    State(state): State<AppState>,
    Form(form): Form<ProfileForm>,
) -> impl IntoResponse {
    let potentially_updated_user = User::update_profile(
        &state,
        &claims.user_id(),
        &form.name,
        &form.birthday,
        &form.avatar_url,
        &form.mail,
    )
    .await;

    match potentially_updated_user {
        Ok(updated_user) => {
            let id_token = JsonWebToken::for_authenticator(&state).generate_id_token(&updated_user);

            if let Ok((id_token, claims)) = id_token {
                (
                    cookies.add(Cookie::new("session_id", id_token)),
                    ProfilePage::from(&state, claims, Some(updated_user), MessageBlock::empty())
                        .await,
                )
                    .into_response()
            } else {
                ProfilePage::from(&state, claims, None, MessageBlock::empty())
                    .await
                    .into_response()
            }
        }
        Err(error) => ProfilePage::from(
            &state,
            claims,
            None,
            MessageBlock::closeable(Level::Error, "", &error.to_string()),
        )
        .await
        .into_response(),
    }
}

#[derive(Deserialize)]
pub struct PasswordForm {
    password: String,
    confirm_password: String,
}

/// Profile update handler
pub async fn update_password_handler(
    claims: IdTokenClaims,
    State(state): State<AppState>,
    Form(form): Form<PasswordForm>,
) -> Result<MessageBlock, MessageBlock> {
    let _ = User::update_password(
        &state,
        &claims.user_id(),
        &form.password,
        &form.confirm_password,
    )
    .await
    .map_err(|error| MessageBlock::closeable(Level::Error, "", &error.to_string()))?;

    Ok(MessageBlock::closeable(
        Level::Success,
        "",
        "Votre password a bien été modifié",
    ))
}
