use askama_axum::{IntoResponse, Template};
use axum::{
    extract::{Query, State},
    Form,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::{
    apps::App,
    users::{confirm::EmailConfirmation, User, UserError},
    AppState,
};

use super::create_session_from_credentials_and_redirect;

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "auth/signup.html")]
pub struct PageTemplate {
    name: String,
    birthday: String,
    email: String,
    error: String,
    app: App,
}

impl PageTemplate {
    /// Generate page from data
    pub fn from(
        name: Option<String>,
        birthday: Option<String>,
        email: Option<String>,
        app: App,
        error: Option<UserError>,
    ) -> Self {
        PageTemplate {
            error: error.map_or("".to_owned(), |error| error.to_string()),
            name: name.unwrap_or("".to_owned()),
            birthday: birthday.unwrap_or("".to_owned()),
            email: email.unwrap_or("".to_owned()),
            app,
        }
    }

    /// Generate page from query params
    pub fn from_query(params: QueryParams, app: App) -> Self {
        Self::from(
            params.name,
            params.birthday,
            params.email,
            app,
            params.error,
        )
    }
}

/// Query parameters definition
/// HTTP parameters used for the get Handler
#[derive(Deserialize)]
pub struct QueryParams {
    name: Option<String>,
    birthday: Option<String>,
    email: Option<String>,
    app_id: Option<i32>,
    error: Option<UserError>,
}

/// Get handler
/// Returns the page using the dedicated HTML template
pub async fn get(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    // Get app to connect to
    let app = App::select_app_or_authenticator(
        &state,
        params.app_id.unwrap_or(state.authenticator_app.id),
    )
    .await;

    PageTemplate::from_query(params, app)
}

/// Signup form
/// Data expected from the signup form in order to create the user
#[derive(Deserialize)]
pub struct SignupForm {
    name: String,
    birthday: String,
    email: String,
    password: String,
    confirm_password: String,
    app_id: i32,
}

/// Post handler
/// Process the signup form to create the user and send a confirmation email
pub async fn post(
    cookies: CookieJar,
    State(state): State<AppState>,
    Form(form): Form<SignupForm>,
) -> Result<impl IntoResponse, PageTemplate> {
    // Get App
    let app = App::select_app_or_authenticator(&state, form.app_id).await;

    // Create user and get user_id generated
    let user = User::create(
        &state,
        &form.name,
        &form.birthday,
        &form.email,
        &form.password,
        &form.confirm_password,
    )
    .await
    .map_err(|error| {
        PageTemplate::from(
            Some(form.name.clone()),
            Some(form.birthday.clone()),
            Some(form.email.clone()),
            app.clone(),
            Some(error),
        )
    })?;

    // Send confirmation email
    let _ = EmailConfirmation::from(&state, user.clone(), app.clone()).send();

    // Connect the user and redirect
    if let Ok(response) = create_session_from_credentials_and_redirect(
        cookies,
        &state,
        &form.email,
        &form.password,
        app.id,
        None,
    )
    .await
    {
        Ok(response.into_response())
    } else {
        Ok(app.redirect_to().into_response())
    }
}
