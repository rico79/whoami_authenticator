use askama_axum::Template;
use axum::extract::{Query, State};
use serde::Deserialize;
use tracing::error;

use crate::{apps::App, auth::IdTokenClaims, AppState};

use super::{User, UserError};

/// Confirm actions
#[derive(Debug, Deserialize)]
pub enum Action {
    Send,
    Confirm,
}

/// Email confirmation struct
#[derive(Clone, Debug)]
pub struct EmailConfirmation {
    state: AppState,
    user: User,
    app: App,
}

impl EmailConfirmation {
    /// Create from User
    pub fn from(state: &AppState, user: User, app: App) -> Self {
        EmailConfirmation { state: state.clone(), user, app }
    }

    /// Email confirmation sending
    /// Send the confirmation email to the user with the confirmation link to click (which will be handled by the get)
    pub fn send_url(&self) -> String {
        format!(
            "{}/confirm?action={:?}&app_id={}&user_id={}",
            self.state.authenticator_app.base_url,
            Action::Send,
            self.app.id,
            self.user.id,
        )
    }

    /// Email confirmation sending
    /// Send the confirmation email to the user with the confirmation link to click (which will be handled by the get)
    pub fn send(&self) -> Result<(), UserError> {
        // Generate code
        let token = IdTokenClaims::new(
            self.user.id.clone(),
            self.user.name.clone(),
            self.user.email.clone(),
            604800, // 604800 seconds = 1 Week
        )
        .encode(self.state.authenticator_app.jwt_secret.clone())
        .map_err(|_| UserError::EmailConfirmationFailed)?;

        // Prepare email
        let validation_url = format!(
            "{}/confirm?action={:?}&app_id={}&token={}",
            self.state.authenticator_app.base_url,
            Action::Confirm,
            self.app.id,
            token
        );
        let subject = "Confirmez votre inscription".to_owned();
        let body = format!("Bonjour {},
        
Vous venez de vous inscrire à l'une des app de Brouclean Softwares: {}
Nous vous souhaitons la bienvenue.

Pour pouvoir continuer et utiliser nos app, veuillez confirmer votre mail en cliquant sur le lien suivant :
{}

Notez que ce code n'est valable qu'une semaine.

En vous souhaitant une excellente journée !!

L'équipe de Brouclean Softwares",self.user.name, self.app.name,validation_url);

        // Send email
        if let Err(error) = self.state.mailer.send(
            format!("{} <{}>", self.user.name, self.user.email),
            subject,
            body,
        ) {
            error!(
                "Could not send email confirmation to '{}' due to : {:?}",
                self.user.email, error
            );

            Err(UserError::EmailConfirmationFailed)
        } else {
            Ok(())
        }
    }
}

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "users/confirm.html")]
pub struct PageTemplate {
    action: String,
    message: (String, String, String),
    app: App,
}

impl PageTemplate {
    fn from(action: Action, app: App, email: Option<String>) -> Self {
        match (action, email) {
            (Action::Send, Some(email)) => PageTemplate {
                action: "Envoi de l'email de confirmation".to_owned(),
                message: (
                    "success".to_owned(),
                    "Email envoyé".to_owned(),
                    format!(
                        "Un email pour confirmation à bien été envoyé à: {}",
                        email.clone()
                    ),
                ),
                app,
            },
            (Action::Send, None) => PageTemplate {
                action: "Envoi de l'email de confirmation".to_owned(),
                message: (
                    "negative".to_owned(),
                    "Envoi impossible".to_owned(),
                    "Veillez réessayer plus tard".to_owned(),
                ),
                app,
            },
            (Action::Confirm, Some(email)) => PageTemplate {
                action: "Confirmation de l'email".to_owned(),
                message: (
                    "success".to_owned(),
                    "Email confirmé".to_owned(),
                    format!("Vous avez bien confirmé le mail suivant: {}", email.clone()),
                ),
                app,
            },
            (Action::Confirm, None) => PageTemplate {
                action: "Confirmation de l'email".to_owned(),
                message: (
                    "negative".to_owned(),
                    "Confirmation impossible".to_owned(),
                    "Le code de confirmation est inconnu".to_owned(),
                ),
                app,
            },
        }
    }
}

/// Query parameters definition
/// HTTP parameters used for the get Handler
#[derive(Deserialize)]
pub struct QueryParams {
    action: Action,
    app_id: String,
    token: Option<String>,
    user_id: Option<String>,
}

/// Get handler
/// Returns the page using the dedicated HTML template
pub async fn get(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Result<PageTemplate, PageTemplate> {
    // Get the app
    let app = App::select_app_or_authenticator(&state, &params.app_id);

    match params.action {
        // Send email
        Action::Send => {
            // Get the user
            let user = User::select_from_id(&state, &params.user_id.unwrap_or_default())
                .await
                .map_err(|_| PageTemplate::from(Action::Send, app.clone(), None))?;

            // Send and check result
            match EmailConfirmation::from(&state, user.clone(), app.clone()).send() {
                Ok(_) => Ok(PageTemplate::from(Action::Send, app, Some(user.email))),
                Err(_) => Err(PageTemplate::from(Action::Send, app, None)),
            }
        }

        // Confirm email
        Action::Confirm => {
            // Confirm email into database and get email confirmed
            let email_confirmed = User::confirm_email(&state, &params.token.unwrap_or_default())
                .await
                .map_err(|_| PageTemplate::from(Action::Confirm, app.clone(), None))?;

            Ok(PageTemplate::from(
                Action::Confirm,
                app,
                Some(email_confirmed),
            ))
        }
    }
}
