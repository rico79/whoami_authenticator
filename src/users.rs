pub mod confirm;
pub mod profile;

use std::fmt;

use serde::Deserialize;
use sqlx::{
    postgres::PgRow,
    types::{time::OffsetDateTime, Uuid},
    FromRow, PgPool, Row,
};
use tracing::log::error;

use crate::{
    auth::IdTokenClaims,
    utils::{crypto::encrypt_text, date_time::DateTime},
    AppState,
};

/// Error types
#[derive(Debug, Deserialize)]
pub enum UserError {
    DatabaseError,
    CryptoError,
    MissingInformation,
    PasswordsDoNotMatch,
    AlreadyExisting,
    InvalidId,
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
        };

        write!(f, "{}", message)
    }
}

/// User struct
#[derive(Clone, Debug)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub email_confirmed: bool,
    pub created_at: DateTime,
}

/// To get User from database
impl FromRow<'_, PgRow> for User {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            email: row.try_get("email")?,
            email_confirmed: row.try_get("email_confirmed")?,
            created_at: DateTime::from(row.try_get::<OffsetDateTime, &str>("created_at")?),
        })
    }
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

    /// Get user data
    /// Get user email
    /// Return the User
    pub async fn select_from_email(
        db_pool: &PgPool,
        user_mail: &String,
    ) -> Result<Self, UserError> {
        // Get user from database
        let user: User = sqlx::query_as(
            "SELECT
                    id,
                    name, 
                    email, 
                    email_confirmed, 
                    created_at 
                FROM 
                    users 
                WHERE 
                    email = $1",
        )
        .bind(user_mail)
        .fetch_one(db_pool)
        .await
        .map_err(|error| {
            error!("Selecting user from email {} -> {:?}", user_mail, error);
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
        email: &String,
    ) -> Result<Self, UserError> {
        // Update ang get user from database
        let user: User = sqlx::query_as(
            "UPDATE users 
                SET 
                    name = $1, 
                    email = $2, 
                    email_confirmed = (email=$2 AND email_confirmed) 
                WHERE 
                    id = $3 
                RETURNING 
                    id,
                    name, 
                    email, 
                    email_confirmed, 
                    created_at",
        )
        .bind(name)
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
    /// Get name, email, password and password confirmation
    /// Return user_id
    pub async fn create(
        state: &AppState,
        name: &String,
        email: &String,
        password: &String,
        confirm_password: &String,
    ) -> Result<Self, UserError> {
        // Check if missing data
        if name.is_empty() || email.is_empty() || password.is_empty() || confirm_password.is_empty()
        {
            return Err(UserError::MissingInformation);
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
                email, 
                encrypted_password) 
            VALUES ($1, $2, $3) 
            RETURNING 
                id,
                name, 
                email, 
                email_confirmed, 
                created_at",
        )
        .bind(name)
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
