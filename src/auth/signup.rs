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

use super::IdSession;

#[derive(Template)]
#[template(path = "auth/signup_page.html")]
pub struct SignupPage {
    name: String,
    birthday: String,
    mail: String,
    app: App,
    requested_endpoint: String,
    message: MessageBlock,
    signin_link: String,
}

impl SignupPage {
    pub fn from(
        name: Option<String>,
        birthday: Option<String>,
        mail: Option<String>,
        requested_endpoint: Option<String>,
        app: App,
        message: MessageBlock,
    ) -> Self {
        SignupPage {
            name: name.unwrap_or("".to_owned()),
            birthday: birthday.unwrap_or("".to_owned()),
            mail: mail.unwrap_or("".to_owned()),
            app: app.clone(),
            requested_endpoint: requested_endpoint.clone().unwrap_or("".to_owned()),
            message,
            signin_link: format!(
                "/signin?app_id={}&requested_endpoint={}",
                app.id,
                url_escape::encode_component(&requested_endpoint.unwrap_or("".to_owned()))
            ),
        }
    }

    pub fn from_query(params: QueryParams, app: App) -> Self {
        Self::from(
            params.name,
            params.birthday,
            params.mail,
            params.requested_endpoint,
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
    requested_endpoint: Option<String>,
}

pub async fn get_handler(
    id_session: Option<IdSession>,
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    let app_to_connect_to = App::select_app_or_authenticator(
        &state,
        params.app_id.unwrap_or(state.authenticator_app.id),
    )
    .await;

    let already_connected = id_session.is_some();

    if already_connected {
        app_to_connect_to.redirect_to_endpoint(None).into_response()
    } else {
        SignupPage::from_query(params, app_to_connect_to).into_response()
    }
}

#[derive(Deserialize)]
pub struct SignupForm {
    name: String,
    birthday: String,
    mail: String,
    password: String,
    confirm_password: String,
    app_id: i32,
    requested_endpoint: Option<String>,
}

pub async fn post_handler(
    cookies: CookieJar,
    State(state): State<AppState>,
    Form(form): Form<SignupForm>,
) -> Result<impl IntoResponse, SignupPage> {
    let app = App::select_app_or_authenticator(&state, form.app_id).await;

    let created_user = User::create(
        &state.db_pool,
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
            form.requested_endpoint.clone(),
            app.clone(),
            MessageBlock::new(Level::Error, "Inscription impossible", &error.to_string()),
        )
    })?;

    let _ = ConfirmationMail::from(&state, created_user.clone(), app.clone()).send();

    if let Ok(redirect_with_session) = IdSession::set_with_redirect_to_endpoint(
        cookies,
        &state,
        &created_user,
        form.requested_endpoint,
    ) {
        Ok(redirect_with_session.into_response())
    } else {
        Ok(app.redirect_to().into_response())
    }
}
