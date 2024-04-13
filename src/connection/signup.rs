use askama_axum::{IntoResponse, Template};
use axum::{extract::State, response::Redirect, Form};
use serde::Deserialize;
use sqlx::Row;

use crate::AppState;

#[derive(Template)]
#[template(path = "connect/signup.html")]
pub struct PageTemplate {}

// Signup form page
pub async fn get() -> impl IntoResponse {
    PageTemplate {}
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
            Err(error) => match error {
                sqlx::Error::Database(database_error) => {
                    // If it is a unique violation error, it means that the user mail already existed
                    if database_error.is_unique_violation() {
                        Redirect::to(&format!("/signup?error={:?}", Error::AlreadyExistingUser))
                    } else {
                        Redirect::to(&format!("/signup?error={:?}", Error::Database(sqlx::Error::Database(database_error))))
                    }
                }
                _ => Redirect::to(&format!("/signup?error={:?}", Error::Database(error))),
            },
        }
    } else {
        Redirect::to(&format!("/signup?error={:?}", Error::InvalidData))
    }
}
