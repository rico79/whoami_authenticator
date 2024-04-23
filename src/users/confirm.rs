use std::collections::HashMap;

use askama_axum::{IntoResponse, Template};
use axum::extract::{Query, State};
use sqlx::{types::Uuid, Row};
use tracing::error;

use crate::AppState;

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "users/confirm.html")]
pub struct PageTemplate {
    email_confirmed: String,
}

/// Get handler
/// Returns the page using the dedicated HTML template
pub async fn get(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // Get confirmation code from query
    let code = params.get("code").unwrap_or(&"".to_owned()).to_string();

    // Check if it is an uuid
    match Uuid::parse_str(&code) {
        Ok(user_id) => {
            // Confirm email into database
            let query_result = sqlx::query(
                "UPDATE users SET email_confirmed = true WHERE user_id = $1 RETURNING email",
            )
            .bind(user_id)
            .fetch_one(&state.db_pool)
            .await;

            // Check the result
            match query_result {
                Ok(row) => PageTemplate {
                    email_confirmed: row.get::<String, &str>("email"),
                },
                Err(err) => {
                    error!(
                        "Impossible to confirm email for code {} due to: '{}'",
                        code, err
                    );
                    PageTemplate {
                        email_confirmed: "".to_owned(),
                    }
                }
            }
        }
        Err(err) => {
            error!(
                "Impossible to confirm email for code {} due to: '{}'",
                code, err
            );
            PageTemplate {
                email_confirmed: "".to_owned(),
            }
        }
    }
}

/// Email confirmation sending
/// Send the confirmation email to the user with the confirmation link to click (which will be handled by the get)
pub fn send_confirmation_email(
    state: &AppState,
    user_name: &String,
    user_email: &String,
    user_id: &String,
) {
    // Prepare email
    let validation_url = format!("{}/confirm?code={}", state.app_url, user_id);
    let subject = "Confirmez votre inscription".to_owned();
    let body = format!("Bonjour {},
        
Vous venez de vous inscrire à l'une des app de Brouclean Softwares.
Nous vous souhaitons la bienvenue.

Pour pouvoir continuer et utiliser nos app, veuillez confirmer votre mail en cliquant sur le lien suivant :
{}

En vous souhaitant une excellente journée !!

L'équipe de Brouclean Softwares",user_name,validation_url);

    // Send email
    if let Err(error) = state
        .mailer
        .send(format!("{} <{}>", user_name, user_email), subject, body)
    {
        error!(
            "Could not send email confirmation to '{}' due to : {:?}",
            user_email, error
        );
    }
}
