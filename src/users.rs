pub mod confirm;
pub mod profile;

use std::fmt;

use serde::Deserialize;
use sqlx::{
    types::chrono::{DateTime, Local, NaiveDate},
    types::Uuid,
    FromRow,
};
use tracing::log::error;

use crate::{utils::{crypto::{encrypt_text, verify_encrypted_text}, jwt::IdTokenClaims}, AppState};

#[derive(Debug, Deserialize)]
pub enum UserError {
    DatabaseError,
    CryptoError,
    MissingInformation,
    PasswordsDoNotMatch,
    AlreadyExistingMail,
    InvalidId,
    InvalidBirthday,
    NotFound,
    MailConfirmationFailed,
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            UserError::DatabaseError => "Un problème est survenu, veuillez réessayer plus tard",
            UserError::CryptoError => "Un problème est survenu, veuillez réessayer plus tard",
            UserError::MissingInformation => "Veuillez remplir toutes les informations",
            UserError::PasswordsDoNotMatch => "Veuillez taper le même mot de passe",
            UserError::AlreadyExistingMail => "Un utilisateur a déjà ce mail",
            UserError::InvalidId => "L'identifiant de l'utilisateur est invalide",
            UserError::NotFound => "L'utilisateur est introuvable",
            UserError::MailConfirmationFailed => "La confirmation du a échouée",
            UserError::InvalidBirthday => "La date de naissance est incorrecte",
        };

        write!(f, "{}", message)
    }
}

#[derive(Clone, Debug, FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub birthday: NaiveDate,
    pub avatar_url: String,
    pub mail: String,
    pub mail_is_confirmed: bool,
    pub created_at: DateTime<Local>,
    encrypted_password: String,
}

impl User {
    pub fn password_match(&self, password: String) -> Result<bool, UserError> {
        verify_encrypted_text(&password, &self.encrypted_password).map_err(|error| {
            error!("{:?}", error);
            UserError::CryptoError
        })
    }

    pub async fn select_from_id(state: &AppState, user_id: Uuid) -> Result<Self, UserError> {
        let user: User = sqlx::query_as(
            "SELECT 
                    id,
                    name, 
                    birthday,
                    avatar_url,
                    mail, 
                    mail_is_confirmed, 
                    created_at,
                    encrypted_password 
                FROM users 
                WHERE 
                    id = $1",
        )
        .bind(user_id)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|error| {
            error!("Selecting user from id {} -> {:?}", user_id, error);
            UserError::NotFound
        })?;

        Ok(user)
    }

    pub async fn select_from_mail(
        state: &AppState,
        mail: &String,
    ) -> Result<Self, UserError> {
        let user: User = sqlx::query_as(
            "SELECT 
                    id,
                    name, 
                    birthday,
                    avatar_url,
                    mail, 
                    mail_is_confirmed, 
                    created_at,
                    encrypted_password 
                FROM users 
                WHERE 
                    mail = $1",
        )
        .bind(mail)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|error| {
            error!("Selecting user from mail {} -> {:?}", mail, error);
            UserError::NotFound
        })?;

        Ok(user)
    }

    pub async fn update_profile(
        state: &AppState,
        user_id: &Uuid,
        name: &String,
        birthday: &String,
        avatar_url: &String,
        mail: &String,
    ) -> Result<Self, UserError> {
        let birthday_date = NaiveDate::parse_from_str(birthday, "%Y-%m-%d").map_err(|error| {
            error!("Updating user {} -> {:?}", name, error);
            UserError::InvalidBirthday
        })?;

        let now = Local::now().date_naive();

        if birthday_date > now {
            return Err(UserError::InvalidBirthday);
        }

        let user: User = sqlx::query_as(
            "UPDATE users 
                SET 
                    name = $1, 
                    birthday = $2,
                    avatar_url = $3,
                    mail = $4, 
                    mail_is_confirmed = (mail=$4 AND mail_is_confirmed) 
                WHERE 
                    id = $5 
                RETURNING 
                    id,
                    name, 
                    birthday,
                    avatar_url,
                    mail, 
                    mail_is_confirmed, 
                    created_at,
                    encrypted_password",
        )
        .bind(name)
        .bind(birthday_date)
        .bind(avatar_url)
        .bind(mail)
        .bind(user_id)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::Database(error) => {
                if error.is_unique_violation() {
                    UserError::AlreadyExistingMail
                } else {
                    error!("Updating profile of {} -> {:?}", user_id, error);
                    UserError::DatabaseError
                }
            }
            _ => {
                error!("Updating profile of {} -> {:?}", user_id, error);
                UserError::DatabaseError
            }
        })?;

        Ok(user)
    }

    pub async fn update_password(
        state: &AppState,
        user_id: &Uuid,
        password: &String,
        confirm_password: &String,
    ) -> Result<bool, UserError> {
        if password.is_empty() || confirm_password.is_empty() {
            return Err(UserError::MissingInformation);
        }

        if password != confirm_password {
            return Err(UserError::PasswordsDoNotMatch);
        }

        let encrypted_password = encrypt_text(password).map_err(|error| {
            error!("Updating password for user {} -> {:?}", user_id, error);
            UserError::CryptoError
        })?;

        let nb_of_password_updated =
            sqlx::query("UPDATE users SET encrypted_password = $1 WHERE id = $2")
                .bind(encrypted_password)
                .bind(user_id)
                .execute(&state.db_pool)
                .await
                .map_err(|error| {
                    error!("Updating password for user {} -> {:?}", user_id, error);
                    UserError::NotFound
                })?
                .rows_affected();

        Ok(nb_of_password_updated > 0)
    }

    pub async fn confirm_mail(state: &AppState, token: &String) -> Result<String, UserError> {
        let claims = IdTokenClaims::decode(
            token.to_string(),
            state.authenticator_app.jwt_secret.clone(),
        )
        .map_err(|error| {
            error!("Confirming mail for token {} -> {:?}", token, error);
            UserError::MailConfirmationFailed
        })?;

        let (confirmed_mail, mail_is_confirmed): (String, bool) = sqlx::query_as(
            "UPDATE users 
            SET 
                mail_is_confirmed = true 
            WHERE 
                id = $1 
                and mail = $2 
            RETURNING 
                mail, 
                mail_is_confirmed",
        )
        .bind(claims.user_id())
        .bind(&claims.mail)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|error| {
            error!("Confirming mail for mail {} -> {:?}", claims.mail, error);
            UserError::NotFound
        })?;

        if mail_is_confirmed {
            Ok(confirmed_mail)
        } else {
            Err(UserError::MailConfirmationFailed)
        }
    }

    pub async fn create(
        state: &AppState,
        name: &String,
        birthday: &String,
        mail: &String,
        password: &String,
        confirm_password: &String,
    ) -> Result<Self, UserError> {
        if name.is_empty()
            || birthday.is_empty()
            || mail.is_empty()
            || password.is_empty()
            || confirm_password.is_empty()
        {
            return Err(UserError::MissingInformation);
        }

        let birthday_date = NaiveDate::parse_from_str(birthday, "%Y-%m-%d").map_err(|error| {
            error!("Creating user {} -> {:?}", name, error);
            UserError::InvalidBirthday
        })?;

        let now = Local::now().date_naive();

        if birthday_date > now {
            return Err(UserError::InvalidBirthday);
        }

        if password != confirm_password {
            return Err(UserError::PasswordsDoNotMatch);
        }

        let encrypted_password = encrypt_text(password).map_err(|error| {
            error!("Creating user {} -> {:?}", name, error);
            UserError::CryptoError
        })?;

        let created_user: User = sqlx::query_as(
            "INSERT INTO users (
                name, 
                birthday, 
                mail,
                encrypted_password,
                avatar_url) 
            VALUES ($1, $2, $3, $4, '') 
            RETURNING 
                id,
                name, 
                birthday,
                avatar_url,
                mail, 
                mail_is_confirmed, 
                created_at,
                encrypted_password",
        )
        .bind(name)
        .bind(birthday_date)
        .bind(mail)
        .bind(encrypted_password)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::Database(error) => {
                if error.is_unique_violation() {
                    UserError::AlreadyExistingMail
                } else {
                    error!("Creating user {} -> {:?}", name, error);
                    UserError::DatabaseError
                }
            }
            _ => {
                error!("Creating user {} -> {:?}", name, error);
                UserError::DatabaseError
            }
        })?;

        Ok(created_user)
    }
}
