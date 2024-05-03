use askama_axum::{IntoResponse, Template};
use axum::{
    extract::{Query, State},
    Form,
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;

use crate::{apps::App, AppState};

use super::{create_session_from_credentials_and_redirect, AuthError, IdTokenClaims};

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "auth/signin.html")]
pub struct PageTemplate {
    error: String,
    mail: String,
    app: App,
    redirect_to: Option<String>,
}

impl PageTemplate {
    /// Generate page from data
    pub fn from(
        state: &AppState,
        mail: Option<String>,
        app: Option<App>,
        error: Option<AuthError>,
        redirect_to: Option<String>,
    ) -> Self {
        // Prepare error message
        let error_message = match error {
            None => "".to_owned(),
            Some(AuthError::InvalidToken) => "".to_owned(),
            Some(AuthError::UserNotExisting) => "Utilisateur inconnu".to_owned(),
            Some(AuthError::WrongCredentials) => {
                "Les données de connexion sont incorrectes".to_owned()
            }
            Some(AuthError::MissingCredentials) => {
                "Veuillez remplir votre mail et votre mot de passe".to_owned()
            }
            _ => "Un problème est survenu, veuillez réessayer plus tard".to_owned(),
        };

        PageTemplate {
            error: error_message,
            mail: mail.unwrap_or("".to_owned()),
            redirect_to,
            app: app.unwrap_or(state.authenticator_app.clone()),
        }
    }

    /// Generate page from query params
    pub fn from_query(state: &AppState, params: QueryParams, app: Option<App>) -> Self {
        Self::from(state, params.mail, app, params.error, params.redirect_to)
    }
}

/// Query parameters definition
/// HTTP parameters used for the get Handler
#[derive(Deserialize)]
pub struct QueryParams {
    mail: Option<String>,
    app_id: Option<i32>,
    error: Option<AuthError>,
    redirect_to: Option<String>,
}

/// Get handler
/// Returns the page using the dedicated HTML template
pub async fn get(
    cookies: CookieJar,
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> impl IntoResponse {
    // Get app to connect to
    let app = App::select_app_or_authenticator(
        &state,
        params.app_id.unwrap_or(state.authenticator_app.id),
    )
    .await;

    // Check if already connected
    if IdTokenClaims::get_from_cookies(&state, &cookies).is_ok() {
        app.redirect_to_another_endpoint(params.redirect_to)
            .into_response()
    } else {
        PageTemplate::from_query(&state, params, Some(app)).into_response()
    }
}

/// Signin form
/// Data expected from the signin form in order to connect the user
#[derive(Deserialize)]
pub struct SigninForm {
    mail: String,
    app_id: i32,
    password: String,
    redirect_to: Option<String>,
}

/// Post handler
/// Process the signin form to create a user session and redirect to the expected app
/// If errors stay in page and alert with errors
pub async fn post(
    cookies: CookieJar,
    State(state): State<AppState>,
    Form(form): Form<SigninForm>,
) -> Result<impl IntoResponse, PageTemplate> {
    // Get App
    let app = App::select_app_or_authenticator(&state, form.app_id).await;

    // Create session
    create_session_from_credentials_and_redirect(
        cookies,
        &state,
        &form.mail,
        &form.password,
        form.app_id,
        form.redirect_to.clone(),
    )
    .await
    .map_err(|error| {
        PageTemplate::from(
            &state,
            Some(form.mail),
            Some(app),
            Some(error),
            form.redirect_to,
        )
    })
}
