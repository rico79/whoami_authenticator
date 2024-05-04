use askama_axum::{IntoResponse, Template};
use axum::{
    extract::{Query, State},
    Form,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
    apps::App,
    general::message::{Level, MessageBlock},
    users::{confirm::ConfirmationMail, User},
    AppState,
};

use super::new_session_into_response;

#[derive(Template)]
#[template(path = "auth/signup_page.html")]
pub struct SignupPage {
    name: String,
    birthday: String,
    mail: String,
    app: App,
    message: MessageBlock,
}

impl SignupPage {
    pub fn from(
        name: Option<String>,
        birthday: Option<String>,
        mail: Option<String>,
        app: App,
        message: MessageBlock,
    ) -> Self {
        SignupPage {
            name: name.unwrap_or("".to_owned()),
            birthday: birthday.unwrap_or("".to_owned()),
            mail: mail.unwrap_or("".to_owned()),
            app,
            message,
        }
    }

    pub fn from_query(params: QueryParams, app: App) -> Self {
        Self::from(
            params.name,
            params.birthday,
            params.mail,
            app,
            MessageBlock::empty(),
        )
    }
}

#[derive(Deserialize)]
pub struct QueryParams {
    name: Option<String>,
    birthday: Option<String>,
    mail: Option<String>,
    app_id: Option<i32>,
}

pub async fn get_handler(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    let app = App::select_app_or_authenticator(
        &state,
        params.app_id.unwrap_or(state.authenticator_app.id),
    )
    .await;

    SignupPage::from_query(params, app)
}

#[derive(Deserialize)]
pub struct SignupForm {
    name: String,
    birthday: String,
    mail: String,
    password: String,
    confirm_password: String,
    app_id: i32,
}

pub async fn post_handler(
    cookies: CookieJar,
    State(state): State<AppState>,
    Form(form): Form<SignupForm>,
) -> Result<impl IntoResponse, SignupPage> {
    let app = App::select_app_or_authenticator(&state, form.app_id).await;

    let created_user = User::create(
        &state,
        &form.name,
        &form.birthday,
        &form.mail,
        &form.password,
        &form.confirm_password,
    )
    .await
    .map_err(|error| {
        SignupPage::from(
            Some(form.name.clone()),
            Some(form.birthday.clone()),
            Some(form.mail.clone()),
            app.clone(),
            MessageBlock::closeable(Level::Error, "Inscription impossible", &error.to_string()),
        )
    })?;

    let _ = ConfirmationMail::from(&state, created_user.clone(), app.clone()).send();

    if let Ok(redirect_with_session) =
        new_session_into_response(cookies, &state, &created_user, &app, None)
    {
        Ok(redirect_with_session.into_response())
    } else {
        Ok(app.redirect_to().into_response())
    }
}
