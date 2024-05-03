use askama_axum::{IntoResponse, Template};
use axum::{
    extract::{Query, State},
    Form,
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;

use crate::{
    apps::App,
    general::message::{Level, MessageBlock},
    AppState,
};

use super::{create_session_from_credentials_and_redirect_response, IdTokenClaims};

#[derive(Template)]
#[template(path = "auth/signin_page.html")]
pub struct SigninPage {
    mail: String,
    app: App,
    redirect_to: Option<String>,
    message: MessageBlock,
}

impl SigninPage {
    pub fn from(
        state: &AppState,
        mail: Option<String>,
        app: Option<App>,
        redirect_to: Option<String>,
        message: MessageBlock,
    ) -> Self {
        SigninPage {
            mail: mail.unwrap_or("".to_owned()),
            redirect_to,
            app: app.unwrap_or(state.authenticator_app.clone()),
            message,
        }
    }

    pub fn from_query(state: &AppState, params: QueryParams, app: Option<App>) -> Self {
        Self::from(
            state,
            params.mail,
            app,
            params.redirect_to,
            MessageBlock::empty(),
        )
    }
}

#[derive(Deserialize)]
pub struct QueryParams {
    mail: Option<String>,
    app_id: Option<i32>,
    redirect_to: Option<String>,
}

pub async fn get_handler(
    cookies: CookieJar,
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    let app_to_connect_to = App::select_app_or_authenticator(
        &state,
        params.app_id.unwrap_or(state.authenticator_app.id),
    )
    .await;

    let already_connected = IdTokenClaims::get_from_cookies(&state, &cookies).is_ok();

    if already_connected {
        app_to_connect_to
            .redirect_to_another_endpoint(params.redirect_to)
            .into_response()
    } else {
        SigninPage::from_query(&state, params, Some(app_to_connect_to)).into_response()
    }
}

#[derive(Deserialize)]
pub struct SigninForm {
    mail: String,
    app_id: i32,
    password: String,
    redirect_to: Option<String>,
}

pub async fn post_handler(
    cookies: CookieJar,
    State(state): State<AppState>,
    Form(form): Form<SigninForm>,
) -> Result<impl IntoResponse, SigninPage> {
    let app_to_connect_to = App::select_app_or_authenticator(&state, form.app_id).await;

    create_session_from_credentials_and_redirect_response(
        cookies,
        &state,
        &form.mail,
        &form.password,
        form.app_id,
        form.redirect_to.clone(),
    )
    .await
    .map_err(|error| {
        SigninPage::from(
            &state,
            Some(form.mail),
            Some(app_to_connect_to),
            form.redirect_to,
            MessageBlock::closeable(Level::Error, "Connexion impossible", &error.to_string()),
        )
    })
}
