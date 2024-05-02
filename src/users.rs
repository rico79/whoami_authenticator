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

use crate::{auth::IdTokenClaims, utils::crypto::encrypt_text, AppState};

/// Error types
#[derive(Debug, Deserialize)]
pub enum UserError {
    DatabaseError,
    CryptoError,
    MissingInformation,
    PasswordsDoNotMatch,
    AlreadyExisting,
    InvalidId,
    InvalidBirthday,
    NotFound,
    EmailConfirmationFailed,
}

// Format Error
impl fmt::Display for UserError {
    // Format
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Define message
        let message = match self {
            UserError::DatabaseError => "Veuillez réessayer plus tard",
            UserError::CryptoError => "Veuillez réessayer plus tard",
            UserError::MissingInformation => "Veuillez remplir toutes les informations",
            UserError::PasswordsDoNotMatch => "Veuillez taper le même mot de passe",
            UserError::AlreadyExisting => "L'utilisateur existe déjà",
            UserError::InvalidId => "L'identifiant de l'utilisateur est invalide",
            UserError::NotFound => "L'utilisateur est introuvable",
            UserError::EmailConfirmationFailed => "La confirmation de l'email a échouée",
            UserError::InvalidBirthday => "La date de naissance est incorrecte",
        };

        write!(f, "{}", message)
    }
}

/// User struct
#[derive(Clone, Debug, FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub birthday: NaiveDate,
    pub avatar_url: String,
    pub email: String,
    pub email_confirmed: bool,
    pub created_at: DateTime<Local>,
}

impl User {
    /// Get user data
    /// Get user id
    /// Return the User
    pub async fn select_from_id(state: &AppState, user_id: Uuid) -> Result<Self, UserError> {
        // Get user from database
        let user: User = sqlx::query_as(
            "SELECT 
                    id,
                    name, 
                    birthday,
                    avatar_url,
                    email, 
                    email_confirmed, 
                    created_at 
                FROM 
                    users 
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

    /// Update user profile
    /// Get user id
    /// Return the User
    pub async fn update_profile(
        state: &AppState,
        user_id: &Uuid,
        name: &String,
        birthday: &String,
        avatar_url: &String,
        email: &String,
    ) -> Result<Self, UserError> {
        // Check if birthday is ok
        let birthday_date = NaiveDate::parse_from_str(birthday, "%Y-%m-%d").map_err(|error| {
            error!("Updating user {} -> {:?}", name, error);
            UserError::InvalidBirthday
        })?;
        if birthday_date > Local::now().date_naive() {
            return Err(UserError::InvalidBirthday);
        }

        // Update ang get user from database
        let user: User = sqlx::query_as(
            "UPDATE users 
                SET 
                    name = $1, 
                    birthday = $2,
                    avatar_url = $3,
                    email = $4, 
                    email_confirmed = (email=$4 AND email_confirmed) 
                WHERE 
                    id = $5 
                RETURNING 
                    id,
                    name, 
                    birthday,
                    avatar_url,
                    email, 
                    email_confirmed, 
                    created_at",
        )
        .bind(name)
        .bind(birthday_date)
        .bind(avatar_url)
        .bind(email)
        .bind(user_id)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|error| {
            error!("Updating profile of {} -> {:?}", user_id, error);
            UserError::NotFound
        })?;

        Ok(user)
    }

    /// Update user password
    /// Get user id
    /// Return the User
    pub async fn update_password(
        state: &AppState,
        user_id: &Uuid,
        password: &String,
        confirm_password: &String,
    ) -> Result<Self, UserError> {
        // Check if missing data
        if password.is_empty() || confirm_password.is_empty() {
            return Err(UserError::MissingInformation);
        }

        // Check if password does not match confirmation
        if password != confirm_password {
            return Err(UserError::PasswordsDoNotMatch);
        }

        // Encrypt the password
        let encrypted_password = encrypt_text(password).map_err(|error| {
            error!("Updating password for user {} -> {:?}", user_id, error);
            UserError::CryptoError
        })?;

        // Update ang get user from database
        let user: User = sqlx::query_as(
            "UPDATE users 
                SET 
                    encrypted_password = $1 
                WHERE 
                    id = $2 
                RETURNING 
                    id,
                    name, 
                    birthday,
                    avatar_url,
                    email, 
                    email_confirmed, 
                    created_at",
        )
        .bind(encrypted_password)
        .bind(user_id)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|error| {
            error!("Updating password for user {} -> {:?}", user_id, error);
            UserError::NotFound
        })?;

        Ok(user)
    }

    /// Confirm email
    /// Get user id
    /// Return email confirmed
    pub async fn confirm_email(state: &AppState, token: &String) -> Result<String, UserError> {
        // Decode token
        let claims = IdTokenClaims::decode(
            token.to_string(),
            state.authenticator_app.jwt_secret.clone(),
        )
        .map_err(|error| {
            error!("Confirming email for token {} -> {:?}", token, error);
            UserError::EmailConfirmationFailed
        })?;

        // Confirm email into database and get email confirmed
        let (email, confirmed): (String, bool) = sqlx::query_as(
            "UPDATE users 
            SET 
                email_confirmed = true 
            WHERE 
                id = $1 
                and email = $2 
            RETURNING 
                email, 
                email_confirmed",
        )
        .bind(claims.user_id())
        .bind(&claims.email)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|error| {
            error!("Confirming email for email {} -> {:?}", claims.email, error);
            UserError::NotFound
        })?;

        // Check if confirmed
        if confirmed {
            Ok(email)
        } else {
            Err(UserError::EmailConfirmationFailed)
        }
    }

    /// Create User from signup
    /// Use the cookies and the App state
    /// Get name, birthday, email, password and password confirmation
    /// Return user_id
    pub async fn create(
        state: &AppState,
        name: &String,
        birthday: &String,
        email: &String,
        password: &String,
        confirm_password: &String,
    ) -> Result<Self, UserError> {
        // Check if missing data
        if name.is_empty()
            || birthday.is_empty()
            || email.is_empty()
            || password.is_empty()
            || confirm_password.is_empty()
        {
            return Err(UserError::MissingInformation);
        }

        // Check if birthday is ok
        let birthday_date = NaiveDate::parse_from_str(birthday, "%Y-%m-%d").map_err(|error| {
            error!("Creating user {} -> {:?}", name, error);
            UserError::InvalidBirthday
        })?;
        if birthday_date > Local::now().date_naive() {
            return Err(UserError::InvalidBirthday);
        }

        // Check if password does not match confirmation
        if password != confirm_password {
            return Err(UserError::PasswordsDoNotMatch);
        }

        // Encrypt the password
        let encrypted_password = encrypt_text(password).map_err(|error| {
            error!("Creating user {} -> {:?}", name, error);
            UserError::CryptoError
        })?;

        // Insert the user
        let user: User = sqlx::query_as(
            "INSERT INTO users (
                name, 
                birthday, 
                email,
                encrypted_password,
                avatar_url) 
            VALUES ($1, $2, $3, $4, '') 
            RETURNING 
                id,
                name, 
                birthday,
                avatar_url,
                email, 
                email_confirmed, 
                created_at",
        )
        .bind(name)
        .bind(birthday_date)
        .bind(email)
        .bind(encrypted_password)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::Database(error) => {
                if error.is_unique_violation() {
                    UserError::AlreadyExisting
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

        // Return user
        Ok(user)
    }
}
