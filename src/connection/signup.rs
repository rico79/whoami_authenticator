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

// User creation
pub async fn post(
    State(state): State<AppState>,
    Form(form): Form<UserCreationForm>,
) -> impl IntoResponse {
    // Insert the user
    let query_result =
        sqlx::query("INSERT INTO users (name, email, password) VALUES ($1, $2, $3) RETURNING id")
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
        Err(error) => Redirect::to(&format!("/signup?error={}", error)),
    }
}
