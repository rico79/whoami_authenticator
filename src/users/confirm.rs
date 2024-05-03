use askama_axum::Template;
use axum::extract::{Query, State};
use serde::Deserialize;
use sqlx::types::Uuid;
use tracing::error;

use crate::{apps::App, auth::IdTokenClaims, general::message::MessageTemplate, AppState};

use super::{User, UserError};

/// Confirm actions
#[derive(Debug, Deserialize)]
pub enum Action {
    Send,
    Confirm,
}

/// Mail confirmation struct
#[derive(Clone, Debug)]
pub struct MailConfirmation {
    state: AppState,
    user: User,
    app: App,
}

impl MailConfirmation {
    /// Create from User
    pub fn from(state: &AppState, user: User, app: App) -> Self {
        MailConfirmation {
            state: state.clone(),
            user,
            app,
        }
    }

    /// Mail confirmation sending
    /// Send the confirmation mail to the user with the confirmation link to click (which will be handled by the get)
    pub fn send_url(&self) -> String {
        format!(
            "{}/confirm?action={:?}&app_id={}&user_id={}",
            self.state.authenticator_app.base_url,
            Action::Send,
            self.app.id,
            self.user.id,
        )
    }

    /// Mail confirmation sending
    /// Send the confirmation mail to the user with the confirmation link to click (which will be handled by the get)
    pub fn send(&self) -> Result<(), UserError> {
        // Generate code
        let token = IdTokenClaims::new(
            &self.state,
            self.user.id,
            self.user.name.clone(),
            self.user.mail.clone(),
            604800, // 604800 seconds = 1 Week
        )
        .encode(self.state.authenticator_app.jwt_secret.clone())
        .map_err(|_| UserError::MailConfirmationFailed)?;

        // Prepare mail
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

        // Send mail
        if let Err(error) = self.state.mailer.send_mail(
            format!("{} <{}>", self.user.name, self.user.mail),
            subject,
            body,
        ) {
            error!(
                "Sending mail confirmation to '{}' -> {:?}",
                self.user.mail, error
            );

            Err(UserError::MailConfirmationFailed)
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
    message: MessageTemplate,
    app: App,
}

impl PageTemplate {
    fn from(action: Action, app: App, mail: Option<String>) -> Self {
        match (action, mail) {
            (Action::Send, Some(mail)) => PageTemplate {
                action: "Envoi de l'mail de confirmation".to_owned(),
                message: MessageTemplate::from(
                    "Mail envoyé".to_owned(),
                    "success".to_owned(),
                    format!(
                        "Un mail pour confirmation à bien été envoyé à: {}",
                        mail.clone()
                    ),
                    false,
                ),
                app,
            },
            (Action::Send, None) => PageTemplate {
                action: "Envoi de l'mail de confirmation".to_owned(),
                message: MessageTemplate::from(
                    "Envoi impossible".to_owned(),
                    "negative".to_owned(),
                    "Veillez réessayer plus tard".to_owned(),
                    false,
                ),
                app,
            },
            (Action::Confirm, Some(mail)) => PageTemplate {
                action: "Confirmation de l'mail".to_owned(),
                message: MessageTemplate::from(
                    "Mail confirmé".to_owned(),
                    "success".to_owned(),
                    format!("Vous avez bien confirmé le mail suivant: {}", mail.clone()),
                    false,
                ),
                app,
            },
            (Action::Confirm, None) => PageTemplate {
                action: "Confirmation de l'mail".to_owned(),
                message: MessageTemplate::from(
                    "Confirmation impossible".to_owned(),
                    "negative".to_owned(),
                    "Le code de confirmation est inconnu".to_owned(),
                    false,
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
    app_id: i32,
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
    let app = App::select_app_or_authenticator(&state, params.app_id).await;

    match (
        params.action,
        Uuid::parse_str(&params.user_id.unwrap_or_default()).ok(),
    ) {
        // Send mail to user
        (Action::Send, Some(user_id)) => {
            // Get the user
            let user = User::select_from_id(&state, user_id)
                .await
                .map_err(|_| PageTemplate::from(Action::Send, app.clone(), None))?;

            // Send and check result
            match MailConfirmation::from(&state, user.clone(), app.clone()).send() {
                Ok(_) => Ok(PageTemplate::from(Action::Send, app, Some(user.mail))),
                Err(_) => Err(PageTemplate::from(Action::Send, app, None)),
            }
        }

        // Send mail to no one
        (Action::Send, None) => Err(PageTemplate::from(Action::Send, app, None)),

        // Confirm mail
        (Action::Confirm, _) => {
            // Confirm mail into database and get mail confirmed
            let confirmed_mail = User::confirm_mail(&state, &params.token.unwrap_or_default())
                .await
                .map_err(|_| PageTemplate::from(Action::Confirm, app.clone(), None))?;

            Ok(PageTemplate::from(
                Action::Confirm,
                app,
                Some(confirmed_mail),
            ))
        }
    }
}
