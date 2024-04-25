use std::collections::HashMap;

use askama_axum::Template;
use axum::extract::{Query, State};
use tracing::error;

use crate::{apps::App, auth::IdTokenClaims, AppState};

use super::{User, UserError};

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
        User::confirm_email(&state, params.get("token").unwrap_or(&"".to_owned())).await
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
pub fn send_confirmation_email(state: &AppState, app: &App, user: &User) -> Result<(), UserError> {
    // Generate code
    let token = IdTokenClaims::new(
        user.id.clone(),
        user.name.clone(),
        user.email.clone(),
        604800, // 604800 seconds = 1 Week
    ).encode(state.jwt_secret.clone()).map_err(|_| UserError::EmailConfirmationFailed)?;

    // Prepare email
    let validation_url = format!("{}/confirm?token={}&app={}", state.app_url, token, app.id);
    let subject = "Confirmez votre inscription".to_owned();
    let body = format!("Bonjour {},
        
Vous venez de vous inscrire à l'une des app de Brouclean Softwares: {}
Nous vous souhaitons la bienvenue.

Pour pouvoir continuer et utiliser nos app, veuillez confirmer votre mail en cliquant sur le lien suivant :
{}

Notez que ce code n'est valable qu'une semaine.

En vous souhaitant une excellente journée !!

L'équipe de Brouclean Softwares",user.name, app.name,validation_url);

    // Send email
    if let Err(error) = state
        .mailer
        .send(format!("{} <{}>", user.name, user.email), subject, body)
    {
        error!(
            "Could not send email confirmation to '{}' due to : {:?}",
            user.email, error
        );

        Err(UserError::EmailConfirmationFailed)
    } else {
        Ok(())
    }
}
