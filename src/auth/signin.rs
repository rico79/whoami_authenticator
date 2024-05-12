use askama_axum::{IntoResponse, Template};
use axum::{
    extract::{Query, State},
    Form,
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;

use crate::{
    apps::App,
    general::{
        message::{Level, MessageBlock},
        AuthenticatorError,
    },
    users::User,
    AppState,
};

use super::{extract_session_claims, redirect_to_app_endpoint_with_new_session_into_response};

#[derive(Template)]
#[template(path = "auth/signin_page.html")]
pub struct SigninPage {
    mail: String,
    app: App,
    requested_endpoint: Option<String>,
    message: MessageBlock,
}

impl SigninPage {
    pub fn for_app_with_redirect_and_message(
        app: App,
        requested_endpoint: Option<String>,
        message: MessageBlock,
    ) -> Self {
        SigninPage {
            mail: "".to_owned(),
            requested_endpoint: requested_endpoint,
            app,
            message,
        }
    }

    pub fn for_app_from_query(app: App, params: QueryParams) -> Self {
        SigninPage {
            mail: params.mail.unwrap_or("".to_owned()),
            requested_endpoint: params.requested_endpoint,
            app,
            message: MessageBlock::empty(),
        }
    }

    pub fn for_app_from_form_with_message(
        app: App,
        form: SigninForm,
        message: MessageBlock,
    ) -> Self {
        SigninPage {
            mail: form.mail,
            requested_endpoint: form.requested_endpoint,
            app,
            message,
        }
    }
}

#[derive(Deserialize)]
pub struct QueryParams {
    mail: Option<String>,
    app_id: Option<i32>,
    requested_endpoint: Option<String>,
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

    let already_connected = extract_session_claims(&state, &cookies).is_ok();

    if already_connected {
        app_to_connect_to
            .redirect_to_endpoint(params.requested_endpoint)
            .into_response()
    } else {
        SigninPage::for_app_from_query(app_to_connect_to, params).into_response()
    }
}

#[derive(Deserialize, Clone)]
pub struct SigninForm {
    mail: String,
    app_id: i32,
    password: String,
    requested_endpoint: Option<String>,
}

pub async fn post_handler(
    cookies: CookieJar,
    State(state): State<AppState>,
    Form(form): Form<SigninForm>,
) -> Result<impl IntoResponse, SigninPage> {
    let app_to_connect = App::select_app_or_authenticator(&state, form.app_id).await;

    let user = User::select_from_mail(&state.db_pool, &form.mail)
        .await
        .map_err(|error| {
            SigninPage::for_app_from_form_with_message(
                app_to_connect.clone(),
                form.clone(),
                MessageBlock::new(Level::Error, "Connexion impossible", &error.to_string()),
            )
        })?;

    let password_is_not_ok = !user
        .password_match(form.password.clone())
        .map_err(|error| {
            SigninPage::for_app_from_form_with_message(
                app_to_connect.clone(),
                form.clone(),
                MessageBlock::new(Level::Error, "Connexion impossible", &error.to_string()),
            )
        })?;

    if password_is_not_ok {
        return Err(SigninPage::for_app_from_form_with_message(
            app_to_connect,
            form,
            MessageBlock::new(
                Level::Error,
                "Connexion impossible",
                &AuthenticatorError::WrongCredentials.to_string(),
            ),
        ));
    }

    redirect_to_app_endpoint_with_new_session_into_response(
        cookies,
        &state,
        &user,
        &app_to_connect,
        form.requested_endpoint.clone(),
    )
    .map_err(|error| {
        SigninPage::for_app_from_form_with_message(
            app_to_connect,
            form,
            MessageBlock::new(Level::Error, "Connexion impossible", &error.to_string()),
        )
    })
}
