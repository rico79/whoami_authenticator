use askama_axum::{IntoResponse, Template};
use axum::{
    extract::{Query, State},
    response::Redirect,
    Form,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::Deserialize;
use sqlx::{types::Uuid, Row};
use tracing::error;

use crate::{crypto::verify_encrypted_text, AppState};

use super::{generate_encoded_jwt, AuthError};

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
pub async fn post(
    cookies: CookieJar,
    State(state): State<AppState>,
    Form(form): Form<SigninForm>,
) -> Result<impl IntoResponse, PageTemplate> {
    // Check if missing credentials
    if form.email.is_empty() || form.password.is_empty() {
        return Err(PageTemplate::from(
            Some(form.email.clone()),
            Some(AuthError::MissingCredentials),
        ));
    }

    // Select the user with this email
    let query_result =
        sqlx::query("SELECT user_id, encrypted_password FROM users WHERE email = $1")
            .bind(&form.email)
            .fetch_optional(&state.db_pool)
            .await
            .map_err(|error| {
                error!(
                    "Signin impossible for user {} due to db error: '{}'",
                    &form.email, error
                );

                PageTemplate::from(Some(form.email.clone()), Some(AuthError::DatabaseError))
            })?;

    // Check if there is a user selected
    if let Some(row) = query_result {
        // Get the user data
        let user_id = row.get::<Uuid, &str>("user_id");
        let encrypted_password = row.get::<String, &str>("encrypted_password");

        // Check password
        if verify_encrypted_text(&form.password, &encrypted_password).map_err(|error| {
            error!(
                "Signin impossible for user {} due to crypto error: '{}'",
                &form.email, error
            );

            PageTemplate::from(Some(form.email.clone()), Some(AuthError::CryptoError))
        })? {
            // Generate JWT
            let jwt = generate_encoded_jwt(user_id.to_string().as_str(), 120, state.jwt_secret)
                .map_err(|error| {
                    error!(
                        "Signin impossible for user {} due to JWT creation error: '{}'",
                        &form.email, error
                    );

                    PageTemplate::from(Some(form.email), Some(AuthError::TokenCreation))
                })?;

            // Redirect with cookie
            Ok((
                cookies.add(Cookie::new("session_id", jwt)),
                Redirect::to("/hello"),
            ))
        }
        // Wrong Password
        else {
            return Err(PageTemplate::from(
                Some(form.email),
                Some(AuthError::WrongCredentials),
            ));
        }
    }
    // No user found
    else {
        return Err(PageTemplate::from(
            Some(form.email),
            Some(AuthError::WrongCredentials),
        ));
    }
}
