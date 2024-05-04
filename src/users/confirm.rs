use std::fmt;

use askama_axum::Template;
use axum::extract::{Query, State};
use serde::Deserialize;
use sqlx::types::Uuid;

use crate::{
    apps::App,
    general::{
        message::{Level, MessageBlock},
        AuthenticatorError,
    },
    utils::jwt::TokenFactory,
    AppState,
};

use super::User;

#[derive(Clone, Debug, Deserialize)]
pub enum Action {
    Sending,
    Confirming,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            Action::Sending => "Envoi du mail de confirmation",
            Action::Confirming => "Confirmation du mail",
        };

        write!(f, "{}", message)
    }
}

#[derive(Template, Clone)]
#[template(path = "users/confirm_page.html")]
pub struct ConfirmPage {
    action: Action,
    message: MessageBlock,
    app: App,
}

#[derive(Deserialize)]
pub struct QueryParams {
    app_id: i32,
    token: Option<String>,
    user_id: Option<String>,
}

pub async fn send_confirm_handler(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Result<ConfirmPage, ConfirmPage> {
    let app = App::select_app_or_authenticator(&state, params.app_id).await;

    let error_response = ConfirmPage {
        action: Action::Sending,
        message: MessageBlock::permanent(
            Level::Error,
            "Envoi impossible",
            "Veillez réessayer plus tard",
        ),
        app: app.clone(),
    };

    let user_id =
        Uuid::parse_str(&params.user_id.unwrap_or_default()).map_err(|_| error_response.clone())?;

    let user = User::select_from_id(&state, user_id)
        .await
        .map_err(|_| error_response.clone())?;

    let successfull_response = ConfirmPage {
        action: Action::Sending,
        message: MessageBlock::permanent(
            Level::Success,
            "Mail envoyé",
            &format!(
                "Un mail pour confirmation à bien été envoyé à: {}",
                user.mail.clone()
            ),
        ),
        app: app.clone(),
    };

    ConfirmationMail::from(&state, user.clone(), app.clone())
        .send()
        .map(|_| successfull_response)
        .map_err(|_| error_response)
}

pub async fn confirm_mail_handler(
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Result<ConfirmPage, ConfirmPage> {
    let app = App::select_app_or_authenticator(&state, params.app_id).await;

    let error_response = ConfirmPage {
        action: Action::Confirming,
        message: MessageBlock::permanent(
            Level::Error,
            "Confirmation impossible",
            "Le code de confirmation est inconnu",
        ),
        app: app.clone(),
    };

    let confirmed_mail = User::confirm_mail(&state, &app, params.token.unwrap_or_default())
        .await
        .map_err(|_| error_response.clone())?;

    Ok(ConfirmPage {
        action: Action::Confirming,
        message: MessageBlock::permanent(
            Level::Success,
            "Mail confirmé",
            &format!(
                "Vous avez bien confirmé le mail suivant: {}",
                confirmed_mail
            ),
        ),
        app,
    })
}

#[derive(Clone, Debug)]
pub struct ConfirmationMail {
    state: AppState,
    user: User,
    app: App,
}

impl ConfirmationMail {
    pub fn from(state: &AppState, user: User, app: App) -> Self {
        Self {
            state: state.clone(),
            user,
            app,
        }
    }

    pub fn send_url(&self) -> String {
        format!(
            "{}/send_confirm?app_id={}&user_id={}",
            self.state.authenticator_app.base_url, self.app.id, self.user.id,
        )
    }

    pub fn send(&self) -> Result<bool, AuthenticatorError> {
        let id_token = TokenFactory::for_app(&self.state, &self.app)
            .generate_id_token(&self.user)
            .map_err(|_| AuthenticatorError::MailConfirmationFailed)?
            .token;

        let validation_url = format!(
            "{}/confirm_mail?app_id={}&token={}",
            self.state.authenticator_app.base_url, self.app.id, id_token
        );

        let mail_subject = "Confirmez votre inscription".to_owned();

        let mail_body = format!("Bonjour {},
        
Vous venez de vous inscrire à l'une des app de Brouclean Softwares: {}
Nous vous souhaitons la bienvenue.

Pour pouvoir continuer et utiliser nos app, veuillez confirmer votre mail en cliquant sur le lien suivant :
{}

Notez que ce code n'est valable qu'une heure.

En vous souhaitant une excellente journée !!

L'équipe de Brouclean Softwares",self.user.name, self.app.name,validation_url);

        self.state.mailer.send_mail(
            format!("{} <{}>", self.user.name, self.user.mail),
            mail_subject,
            mail_body,
        )
    }
}
