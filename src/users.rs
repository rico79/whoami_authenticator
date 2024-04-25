pub mod confirm;
pub mod profile;

use serde::Deserialize;
use sqlx::types::{time::OffsetDateTime, Uuid};

use crate::{
    utils::{crypto::encrypt_text, date_time::DateTime},
    AppState,
};

/// Error types for auth errors
#[derive(Debug, Deserialize)]
pub enum UserError {
    DatabaseError,
    CryptoError,
    MissingInformation,
    PasswordsDoNotMatch,
    AlreadyExistingUser,
    InvalidId,
    UserNotFound,
    EmailConfirmationFailed,
}

/// User struct
#[derive(Clone, Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub email_confirmed: bool,
    pub created_at: DateTime,
}

impl User {
    /// Get user data
    /// Get user id
    /// Return the User
    pub async fn select_from_id(state: &AppState, user_id: &String) -> Result<Self, UserError> {
        // Convert the user id into Uuid
        let user_uuid = Uuid::parse_str(user_id).map_err(|_| UserError::InvalidId)?;

        // Get user from database
        let (name, email, email_confirmed, created_at): (String, String, bool, OffsetDateTime) =
            sqlx::query_as(
                "SELECT name, email, email_confirmed, created_at FROM users WHERE user_id = $1",
            )
            .bind(user_uuid)
            .fetch_one(&state.db_pool)
            .await
            .map_err(|_| UserError::UserNotFound)?;

        Ok(User {
            id: user_id.to_string(),
            name,
            email,
            email_confirmed,
            created_at: DateTime::from(created_at),
        })
    }

    /// Update user profile
    /// Get user id
    /// Return the User
    pub async fn update_profile(state: &AppState, user_id: &String, name: &String, email: &String) -> Result<Self, UserError> {
        // Convert the user id into Uuid
        let user_uuid = Uuid::parse_str(user_id).map_err(|_| UserError::InvalidId)?;

        // Update ang get user from database
        let (name, email, email_confirmed, created_at): (String, String, bool, OffsetDateTime) =
            sqlx::query_as(
                "UPDATE users SET name = $1, email = $2, email_confirmed = (email=$2 AND email_confirmed) WHERE user_id = $3 RETURNING name, email, email_confirmed, created_at",
            )
            .bind(name)
            .bind(email)
            .bind(user_uuid)
            .fetch_one(&state.db_pool)
            .await
            .map_err(|_| UserError::UserNotFound)?;

        Ok(User {
            id: user_id.to_string(),
            name,
            email,
            email_confirmed,
            created_at: DateTime::from(created_at),
        })
    }

    /// Confirm email
    /// Get user id
    /// Return email confirmed
    pub async fn confirm_email(state: &AppState, user_id: &String) -> Result<String, UserError> {
        // Convert the user id into Uuid
        let user_uuid = Uuid::parse_str(user_id).map_err(|_| UserError::InvalidId)?;

        // Confirm email into database and get email confirmed
        let (email, confirmed): (String, bool) = sqlx::query_as(
            "UPDATE users SET email_confirmed = true WHERE user_id = $1 RETURNING email, email_confirmed",
        )
        .bind(user_uuid)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|_| UserError::UserNotFound)?;

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
        let encrypted_password = encrypt_text(password).map_err(|_| UserError::CryptoError)?;

        // Insert the user
        let (user_id, created_at): (Uuid, OffsetDateTime) = sqlx::query_as(
            "INSERT INTO users (name, email, encrypted_password) VALUES ($1, $2, $3) RETURNING user_id, created_at",
        )
        .bind(name)
        .bind(email)
        .bind(encrypted_password)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|error| match error {
            sqlx::Error::Database(error) => {
                if error.is_unique_violation() {
                    UserError::AlreadyExistingUser
                } else {
                    UserError::DatabaseError
                }
            }
            _ => UserError::DatabaseError,
        })?;

        // Return user
        Ok(User {
            id: user_id.to_string(),
            name: name.to_string(),
            email: email.to_string(),
            email_confirmed: false,
            created_at: DateTime::from(created_at),
        })
    }
}
