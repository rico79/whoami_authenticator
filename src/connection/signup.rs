use std::collections::HashMap;

use askama_axum::{IntoResponse, Template};
use axum::{
    extract::{Query, State},
    response::Redirect,
    Form,
};
use serde::Deserialize;
use sqlx::Row;

use crate::AppState;

#[derive(Template)]
#[template(path = "connect/signup.html")]
pub struct PageTemplate {
    error: String,
    name: String,
    email: String,
}

// Signup form page
pub async fn get(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    // Get data from query
    let query_error = params.get("error").unwrap_or(&"".to_owned()).to_string();
    let name = params.get("name").unwrap_or(&"".to_owned()).to_string();
    let email = params.get("email").unwrap_or(&"".to_owned()).to_string();

    // Check error type to choose message to show
    let error = match query_error.as_str() {
        "AlreadyExistingUser" => format!("Le mail {} est déjà utilisé", email),
        "InvalidData" => "Veuillez corriger les informations remplies".to_owned(),
        _ => "Un problème est survenu, veuillez réessayer plus tard".to_owned(),
    };

    PageTemplate { error, name, email }
}

#[derive(Deserialize)]
pub struct UserCreationForm {
    name: String,
    email: String,
    password: String,
    confirm_password: String,
}

// User Creation Errors
#[derive(Debug)]
pub enum Error {
    Database(sqlx::Error),
    InvalidData,
    AlreadyExistingUser,
}

// User creation
pub async fn post(
    State(state): State<AppState>,
    Form(form): Form<UserCreationForm>,
) -> impl IntoResponse {
    // Check if data are correct
    if !&form.name.is_empty()
        && !&form.email.is_empty()
        && !&form.password.is_empty()
        && !&form.confirm_password.is_empty()
        && &form.password == &form.confirm_password
    {
        // Insert the user
        let query_result = sqlx::query(
            "INSERT INTO users (name, email, password) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(&form.name)
        .bind(&form.email)
        .bind(&form.password)
        .fetch_one(&state.db_pool)
        .await;

        // Check the result
        match query_result {
            Ok(row) => {
                let user_id = row.get::<i32, &str>("id");
                Redirect::to(&format!("/hello?name={}", user_id))
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
                    Redirect::to(&format!(
                        "/signup?error={:?}&name={}&email={}",
                        Error::Database(sqlx::Error::Database(database_error)),
                        &form.name,
                        &form.email
                    ))
                }
            }
            Err(error) => Redirect::to(&format!(
                "/signup?error={:?}&name={}&email={}",
                Error::Database(error),
                &form.name,
                &form.email
            )),
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
