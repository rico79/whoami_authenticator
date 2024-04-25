use std::collections::HashMap;

use askama_axum::Template;
use axum::extract::{Query, State};
use tracing::error;

use crate::{apps::App, AppState};

use super::User;

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "users/confirm.html")]
pub struct PageTemplate {
    email_confirmed: Option<String>,
    app: App,
}

/// Get handler
/// Returns the page using the dedicated HTML template
pub async fn get(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> PageTemplate {
    // Get the app related to the confirmation
    let app = App::from_app_id(params.get("app").unwrap_or(&"".to_owned()).to_string());

    // Confirm email into database and get email confirmed
    if let Ok(email_confirmed) =
        User::confirm_email(&state, params.get("code").unwrap_or(&"".to_owned())).await
    {
        PageTemplate {
            email_confirmed: Some(email_confirmed),
            app,
        }
    } else {
        PageTemplate {
            email_confirmed: None,
            app,
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
    app: &App,
) {
    // Prepare email
    let validation_url = format!("{}/confirm?code={}&app={}", state.app_url, user_id, app.id);
    let subject = "Confirmez votre inscription".to_owned();
    let body = format!("Bonjour {},
        
Vous venez de vous inscrire à l'une des app de Brouclean Softwares: {}
Nous vous souhaitons la bienvenue.

Pour pouvoir continuer et utiliser nos app, veuillez confirmer votre mail en cliquant sur le lien suivant :
{}

En vous souhaitant une excellente journée !!

L'équipe de Brouclean Softwares",user_name, app.name,validation_url);

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
