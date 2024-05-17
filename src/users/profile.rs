use askama_axum::{IntoResponse, Template};
use axum::{extract::State, Form};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
    auth::IdSession,
    general::{
        message::{Level, MessageBlock},
        navbar::NavBarBlock,
    },
    AppState,
};

use super::{confirm::ConfirmationMail, User};

#[derive(Template)]
#[template(path = "users/profile_page.html")]
pub struct ProfilePage {
    navbar: NavBarBlock,
    user: Option<User>,
    confirm_send_url: String,
    profile_message: MessageBlock,
    password_message: MessageBlock,
}

impl ProfilePage {
    pub async fn from(
        state: &AppState,
        id_session: IdSession,
        returned_user: Option<User>,
        profile_message: MessageBlock,
    ) -> Self {
        let user = returned_user.or(User::select_from_id(&state.db_pool, id_session.user_id)
            .await
            .ok());

        let confirm_send_url = match &user {
            Some(user) => {
                ConfirmationMail::from(&state, user.clone(), state.authenticator_app.clone())
                    .send_url()
            }
            None => "".to_owned(),
        };

        ProfilePage {
            navbar: NavBarBlock::from(state, Some(id_session)),
            user: user,
            confirm_send_url,
            profile_message,
            password_message: MessageBlock::empty(),
        }
    }
}

pub async fn get_handler(
    id_session: IdSession,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ProfilePage::from(&state, id_session, None, MessageBlock::empty()).await
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
    id_session: IdSession,
    State(state): State<AppState>,
    Form(form): Form<ProfileForm>,
) -> impl IntoResponse {
    let potentially_updated_user = User::update_profile(
        &state.db_pool,
        &id_session.user_id,
        &form.name,
        &form.birthday,
        &form.avatar_url,
        &form.mail,
    )
    .await;

    match potentially_updated_user {
        Ok(updated_user) => {
            match IdSession::set_with_redirect_to_app_endpoint(
                cookies,
                &state,
                &updated_user,
                &state.authenticator_app,
                Some("/profile".to_owned()),
            ) {
                Ok(response) => response.into_response(),

                Err(error) => ProfilePage::from(
                    &state,
                    id_session,
                    Some(updated_user),
                    MessageBlock::new(Level::Error, "", &error.to_string()),
                )
                .await
                .into_response(),
            }
        }

        Err(error) => ProfilePage::from(
            &state,
            id_session,
            None,
            MessageBlock::new(Level::Error, "", &error.to_string()),
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
    id_session: IdSession,
    State(state): State<AppState>,
    Form(form): Form<PasswordForm>,
) -> Result<MessageBlock, MessageBlock> {
    let _ = User::update_password(
        &state.db_pool,
        &id_session.user_id,
        &form.password,
        &form.confirm_password,
    )
    .await
    .map_err(|error| MessageBlock::new(Level::Error, "", &error.to_string()))?;

    Ok(MessageBlock::new(
        Level::Success,
        "",
        "Votre password a bien été modifié",
    ))
}
