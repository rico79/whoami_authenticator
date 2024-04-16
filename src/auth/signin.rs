use std::collections::HashMap;

use askama_axum::{IntoResponse, Template};
use axum::{
    extract::{Query, State},
    response::Redirect,
    Form,
};
use serde::Deserialize;
use sqlx::{types::Uuid, Row};
use tracing::error;

use crate::{
    crypto::{generate_encoded_jwt, verify_encrypted_text},
    AppState,
};

/** Template
 * HTML page definition with dynamic data
 */
#[derive(Template)]
#[template(path = "connection/signin.html")]
pub struct PageTemplate {
    error: String,
    email: String,
}

/** Singin errors
 * List of the different errors that can occur during the signin process
 */
#[derive(Debug)]
pub enum Error {
    DatabaseError,
    CryptoError,
    InvalidData,
    UnregisteredEmail,
    WrongPassword,
}

/** Get handler
 * Returns the page using the dedicated HTML template
 */
pub async fn get(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    // Get data from query
    let query_error = params.get("error").unwrap_or(&"".to_owned()).to_string();
    let email = params.get("email").unwrap_or(&"".to_owned()).to_string();

    // Check error type to choose message to show
    let error = match query_error.as_str() {
        "" => "".to_owned(),
        "UnregisteredEmail" => format!("Le mail {} est inconnu", email),
        "WrongPassword" => format!("Mauvais mot de passe"),
        "InvalidData" => "Veuillez corriger les informations remplies".to_owned(),
        _ => "Un problème est survenu, veuillez réessayer plus tard".to_owned(),
    };

    PageTemplate { error, email }
}

/** Signin form
 * Data expected from the signin form in order to connect the user
 */
#[derive(Deserialize)]
pub struct SigninForm {
    email: String,
    password: String,
}

/** Post handler
 * Process the signin form to create a user session and redirect to the expected app
 */
pub async fn post(
    State(state): State<AppState>,
    Form(form): Form<SigninForm>,
) -> impl IntoResponse {
    // Check if data are correct
    if !&form.email.is_empty() && !&form.password.is_empty() {
        // Insert the user
        let query_result =
            sqlx::query("SELECT user_id, encrypted_password FROM users WHERE email = $1")
                .bind(&form.email)
                .fetch_optional(&state.db_pool)
                .await;

        // Check the result
        match query_result {
            Ok(Some(row)) => {
                // Get the user data
                let user_id = row.get::<Uuid, &str>("user_id");
                let encrypted_password = row.get::<String, &str>("encrypted_password");

                // Check the password
                match verify_encrypted_text(&form.password, &encrypted_password) {
                    // Good password
                    Ok(true) => {
                        // Generate JWT
                        match generate_encoded_jwt(
                            user_id.to_string().as_str(),
                            120,
                            state.jwt_secret,
                        ) {
                            Ok(jwt) => {
                                // Redirect
                                Redirect::to(&format!("/hello?name={}", jwt))
                            }
                            Err(error) => {
                                error!(
                                    "Signin impossible for user {} due to crypto error: '{}'",
                                    &form.email, error
                                );

                                Redirect::to(&format!(
                                    "/signin?error={:?}&email={}",
                                    Error::CryptoError,
                                    &form.email
                                ))
                            }
                        }
                    }
                    // Wrong password
                    Ok(false) => Redirect::to(&format!(
                        "/signin?error={:?}&email={}",
                        Error::WrongPassword,
                        &form.email
                    )),
                    // Crypto error while checking the password
                    Err(error) => {
                        error!(
                            "Signin impossible for user {} due to crypto error: '{}'",
                            &form.email, error
                        );

                        Redirect::to(&format!(
                            "/signin?error={:?}&email={}",
                            Error::CryptoError,
                            &form.email
                        ))
                    }
                }
            }
            Ok(None) => Redirect::to(&format!(
                "/signin?error={:?}&email={}",
                Error::UnregisteredEmail,
                &form.email
            )),
            Err(error) => {
                error!(
                    "Signin impossible for user {} due to db error: '{}'",
                    &form.email, error
                );

                Redirect::to(&format!(
                    "/signin?error={:?}&email={}",
                    Error::DatabaseError,
                    &form.email
                ))
            }
        }
    } else {
        Redirect::to(&format!(
            "/signin?error={:?}&email={}",
            Error::InvalidData,
            &form.email
        ))
    }
}
