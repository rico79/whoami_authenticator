use askama_axum::{IntoResponse, Template};
use axum::{
    extract::{Query, State},
    Form,
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;

use crate::AppState;

use super::{create_session_cookie_from_credentials_and_redirect, AuthError};

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "connection/signin.html")]
pub struct PageTemplate {
    error: String,
    email: String,
}

impl PageTemplate {
    /// Generate page from data
    pub fn from(email: Option<String>, error: Option<AuthError>) -> PageTemplate {
        // Prepare error message
        let error_message = match error {
            None => "".to_owned(),
            Some(AuthError::InvalidToken) => "".to_owned(),
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
            email: email.unwrap_or("".to_owned()),
        }
    }

    /// Generate page from query params
    pub fn from_query(params: QueryParams) -> PageTemplate {
        Self::from(params.email, params.error)
    }
}

/// Query parameters definition
/// HTTP parameters used for the get Handler
#[derive(Deserialize)]
pub struct QueryParams {
    email: Option<String>,
    error: Option<AuthError>,
}

/// Get handler
/// Returns the page using the dedicated HTML template
pub async fn get(Query(params): Query<QueryParams>) -> impl IntoResponse {
    PageTemplate::from_query(params)
}

/// Signin form
/// Data expected from the signin form in order to connect the user
#[derive(Deserialize)]
pub struct SigninForm {
    email: String,
    password: String,
}

/// Post handler
/// Process the signin form to create a user session and redirect to the expected app
/// If errors stay in page and alert with errors
pub async fn post(
    cookies: CookieJar,
    State(state): State<AppState>,
    Form(form): Form<SigninForm>,
) -> Result<impl IntoResponse, PageTemplate> {
    create_session_cookie_from_credentials_and_redirect(
        cookies,
        &state,
        &form.email,
        &form.password,
        "/hello",
    )
    .await
    .map_err(|error| PageTemplate::from(Some(form.email), Some(error)))
}
