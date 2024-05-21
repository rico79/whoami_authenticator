pub mod confirm;
pub mod profile;

use sqlx::{
    types::{
        time::{Date, OffsetDateTime},
        Uuid,
    },
    FromRow, PgPool,
};
use tracing::log::error;

use crate::{
    apps::App,
    general::AuthenticatorError,
    utils::{
        crypto::{encrypt_text, verify_encrypted_text},
        jwt::TokenFactory,
        time::HtmlDate,
    },
    AppState,
};

#[derive(Clone, Debug, FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub birthday: Date,
    pub avatar_url: String,
    pub mail: String,
    pub mail_is_confirmed: bool,
    pub created_at: OffsetDateTime,
    encrypted_password: String,
}

impl User {
    pub fn password_match(&self, password: String) -> Result<bool, AuthenticatorError> {
        verify_encrypted_text(&password, &self.encrypted_password)
    }

    pub async fn select_from_id(
        db_pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Self, AuthenticatorError> {
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
        .fetch_one(db_pool)
        .await
        .map_err(|error| {
            error!("Selecting user from id {} -> {:?}", user_id, error);
            AuthenticatorError::UserNotFound
        })?;

        Ok(user)
    }

    pub async fn select_from_mail(
        db_pool: &PgPool,
        mail: &String,
    ) -> Result<Option<Self>, AuthenticatorError> {
        let user: Option<Self> = sqlx::query_as(
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
        .fetch_optional(db_pool)
        .await
        .map_err(|error| {
            error!("Selecting user from mail {} -> {:?}", mail, error);
            AuthenticatorError::UserNotFound
        })?;

        Ok(user)
    }

    pub async fn update_profile(
        db_pool: &PgPool,
        user_id: &Uuid,
        name: &String,
        birthday: &String,
        avatar_url: &String,
        mail: &String,
    ) -> Result<Self, AuthenticatorError> {
        let birthday_date: Date = HtmlDate::from(birthday).try_into()?;

        let now = OffsetDateTime::now_utc().date();

        if birthday_date > now {
            return Err(AuthenticatorError::InvalidBirthday);
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
        .fetch_one(db_pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::Database(error) => {
                if error.is_unique_violation() {
                    AuthenticatorError::AlreadyExistingMail
                } else {
                    error!("Updating profile of {} -> {:?}", user_id, error);
                    AuthenticatorError::DatabaseError
                }
            }
            _ => {
                error!("Updating profile of {} -> {:?}", user_id, error);
                AuthenticatorError::DatabaseError
            }
        })?;

        Ok(user)
    }

    pub async fn delete(&self, db_pool: &PgPool) -> Result<bool, AuthenticatorError> {
        let query_result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(&self.id)
            .execute(db_pool)
            .await
            .map_err(|error| {
                error!("Deleting profile of {} -> {:?}", self.id, error);
                AuthenticatorError::DatabaseError
            })?;

        Ok(query_result.rows_affected() > 0)
    }

    pub async fn update_password(
        db_pool: &PgPool,
        user_id: &Uuid,
        password: &String,
        confirm_password: &String,
    ) -> Result<bool, AuthenticatorError> {
        if password.is_empty() || confirm_password.is_empty() {
            return Err(AuthenticatorError::MissingInformation);
        }

        if password != confirm_password {
            return Err(AuthenticatorError::PasswordsDoNotMatch);
        }

        let encrypted_password = encrypt_text(password)?;

        let nb_of_password_updated =
            sqlx::query("UPDATE users SET encrypted_password = $1 WHERE id = $2")
                .bind(encrypted_password)
                .bind(user_id)
                .execute(db_pool)
                .await
                .map_err(|error| {
                    error!("Updating password for user {} -> {:?}", user_id, error);
                    AuthenticatorError::UserNotFound
                })?
                .rows_affected();

        Ok(nb_of_password_updated > 0)
    }

    pub async fn confirm_mail(
        state: &AppState,
        app: &App,
        token: String,
    ) -> Result<String, AuthenticatorError> {
        let claims = TokenFactory::for_app(state, app)
            .extract_id_token(token)?
            .claims;

        let (confirmed_mail, mail_is_confirmed): (String, bool) = sqlx::query_as(
            "UPDATE users 
            SET 
                mail_is_confirmed = true 
            WHERE 
                id = $1 
            AND mail = $2 
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
            AuthenticatorError::UserNotFound
        })?;

        if mail_is_confirmed {
            Ok(confirmed_mail)
        } else {
            Err(AuthenticatorError::MailConfirmationFailed)
        }
    }

    pub async fn create(
        db_pool: &PgPool,
        name: &String,
        birthday: &String,
        mail: &String,
        password: &String,
        confirm_password: &String,
    ) -> Result<Self, AuthenticatorError> {
        if name.is_empty()
            || birthday.is_empty()
            || mail.is_empty()
            || password.is_empty()
            || confirm_password.is_empty()
        {
            return Err(AuthenticatorError::MissingInformation);
        }

        let birthday_date: Date = HtmlDate::from(birthday).try_into()?;

        let now = OffsetDateTime::now_utc().date();

        if birthday_date > now {
            return Err(AuthenticatorError::InvalidBirthday);
        }

        if password != confirm_password {
            return Err(AuthenticatorError::PasswordsDoNotMatch);
        }

        let encrypted_password = encrypt_text(password)?;

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
        .fetch_one(db_pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::Database(error) => {
                if error.is_unique_violation() {
                    AuthenticatorError::AlreadyExistingMail
                } else {
                    error!("Creating user {} -> {:?}", name, error);
                    AuthenticatorError::DatabaseError
                }
            }
            _ => {
                error!("Creating user {} -> {:?}", name, error);
                AuthenticatorError::DatabaseError
            }
        })?;

        Ok(created_user)
    }
}
