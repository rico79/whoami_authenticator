use askama_axum::{IntoResponse, Template};
use axum::{
    extract::{Query, State},
    response::Redirect,
    Form,
};
use serde::Deserialize;

use crate::{
    users::{confirm::send_confirmation_email, create_user, UserError},
    AppState,
};

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "connection/signup.html")]
pub struct PageTemplate {
    name: String,
    email: String,
    error: String,
}

impl PageTemplate {
    /// Generate page from data
    pub fn from(
        name: Option<String>,
        email: Option<String>,
        error: Option<UserError>,
    ) -> PageTemplate {
        // Prepare error message
        let error_message = match error {
            None => "".to_owned(),
            Some(UserError::AlreadyExistingUser) => format!(
                "Le mail {} est déjà utilisé",
                email.clone().unwrap_or("".to_owned())
            ),
            Some(UserError::InvalidData) => {
                "Veuillez corriger les informations remplies".to_owned()
            }
            _ => "Un problème est survenu, veuillez réessayer plus tard".to_owned(),
        };

        PageTemplate {
            error: error_message,
            name: name.unwrap_or("".to_owned()),
            email: email.unwrap_or("".to_owned()),
        }
    }

    /// Generate page from query params
    pub fn from_query(params: QueryParams) -> PageTemplate {
        Self::from(params.name, params.email, params.error)
    }
}

/// Query parameters definition
/// HTTP parameters used for the get Handler
#[derive(Deserialize)]
pub struct QueryParams {
    name: Option<String>,
    email: Option<String>,
    error: Option<UserError>,
}

/// Get handler
/// Returns the page using the dedicated HTML template
pub async fn get(Query(params): Query<QueryParams>) -> impl IntoResponse {
    PageTemplate::from_query(params)
}

/// Signup form
/// Data expected from the signup form in order to create the user
#[derive(Deserialize)]
pub struct SignupForm {
    name: String,
    email: String,
    password: String,
    confirm_password: String,
}

/// Post handler
/// Process the signup form to create the user and send a confirmation email
pub async fn post(
    State(state): State<AppState>,
    Form(form): Form<SignupForm>,
) -> Result<impl IntoResponse, PageTemplate> {
    // Create user and get user_id generated
    let user_id = create_user(
        &state,
        &form.name,
        &form.email,
        &form.password,
        &form.confirm_password,
    )
    .await
    .map_err(|error| {
        PageTemplate::from(
            Some(form.name.clone()),
            Some(form.email.clone()),
            Some(error),
        )
    })?;

    // Send confirmation email
    send_confirmation_email(&state, &form.name, &form.email, &user_id);

    // Connect the user and redirect
    Ok(Redirect::to("/welcome"))
}
