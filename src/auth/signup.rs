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

use crate::{crypto::encrypt_text, users::confirmation::send_confirmation_email, AppState};

/** Template
 * HTML page definition with dynamic data
 */
#[derive(Template)]
#[template(path = "connection/signup.html")]
pub struct PageTemplate {
    error: String,
    name: String,
    email: String,
}

/** Singup errors
 * List of the different errors that can occur during the signup process
 */
#[derive(Debug)]
pub enum Error {
    DatabaseError,
    CryptoError,
    InvalidData,
    AlreadyExistingUser,
}

/** Get handler
 * Returns the page using the dedicated HTML template
 */
pub async fn get(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    // Get data from query
    let query_error = params.get("error").unwrap_or(&"".to_owned()).to_string();
    let name = params.get("name").unwrap_or(&"".to_owned()).to_string();
    let email = params.get("email").unwrap_or(&"".to_owned()).to_string();

    // Check error type to choose message to show
    let error = match query_error.as_str() {
        "" => "".to_owned(),
        "AlreadyExistingUser" => format!("Le mail {} est déjà utilisé", email),
        "InvalidData" => "Veuillez corriger les informations remplies".to_owned(),
        _ => "Un problème est survenu, veuillez réessayer plus tard".to_owned(),
    };

    PageTemplate { error, name, email }
}

/** Signup form
 * Data expected from the signup form in order to create the user
 */
#[derive(Deserialize)]
pub struct SignupForm {
    name: String,
    email: String,
    password: String,
    confirm_password: String,
}

/** Post handler
 * Process the signup form to create the user and send a confirmation email
 */
pub async fn post(
    State(state): State<AppState>,
    Form(form): Form<SignupForm>,
) -> impl IntoResponse {
    // Check if data are correct
    if !&form.name.is_empty()
        && !&form.email.is_empty()
        && !&form.password.is_empty()
        && !&form.confirm_password.is_empty()
        && &form.password == &form.confirm_password
    {
        // Encrypt the password
        match encrypt_text(&form.password) {
            Ok(encrypted_password) => {
                // Insert the user
                let query_result = sqlx::query(
                    "INSERT INTO users (name, email, encrypted_password) VALUES ($1, $2, $3) RETURNING user_id",
                )
                .bind(&form.name)
                .bind(&form.email)
                .bind(encrypted_password)
                .fetch_one(&state.db_pool)
                .await;

                // Check the result
                match query_result {
                    Ok(row) => {
                        // Get the user id created
                        let user_id = row.get::<Uuid, &str>("user_id");

                        // Send the email for validation
                        send_confirmation_email(
                            &state,
                            &form.name,
                            &form.email,
                            &user_id.to_string(),
                        );

                        // Redirect
                        Redirect::to(&format!("/hello?name={}", form.name))
                    }
                    Err(sqlx::Error::Database(database_error)) => {
                        // If it is a unique violation error, it means that the user mail already existed
                        if database_error.is_unique_violation() {
                            Redirect::to(&format!(
                                "/signup?error={:?}&name={}&email={}",
                                Error::AlreadyExistingUser,
                                &form.name,
                                &form.email
                            ))
                        } else {
                            error!(
                                "Signup impossible for user {} due to db error: '{}'",
                                &form.email, database_error
                            );

                            Redirect::to(&format!(
                                "/signup?error={:?}&name={}&email={}",
                                Error::DatabaseError,
                                &form.name,
                                &form.email
                            ))
                        }
                    }
                    Err(error) => {
                        error!(
                            "Signup impossible for user {} due to db error: '{}'",
                            &form.email, error
                        );

                        Redirect::to(&format!(
                            "/signup?error={:?}&name={}&email={}",
                            Error::DatabaseError,
                            &form.name,
                            &form.email
                        ))
                    }
                }
            }
            Err(error) => {
                error!(
                    "Signup impossible for user {} due to crypto error: '{}'",
                    &form.email, error
                );

                Redirect::to(&format!(
                    "/signup?error={:?}&name={}&email={}",
                    Error::CryptoError,
                    &form.name,
                    &form.email
                ))
            }
        }
    } else {
        Redirect::to(&format!(
            "/signup?error={:?}&name={}&email={}",
            Error::InvalidData,
            &form.name,
            &form.email
        ))
    }
}
