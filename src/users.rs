use serde::Deserialize;
use sqlx::{types::Uuid, Row};

use crate::{utils::crypto::encrypt_text, AppState};

pub mod confirm;

/// Error types for auth errors
#[derive(Debug, Deserialize)]
pub enum UserError {
    DatabaseError,
    CryptoError,
    MissingInformation,
    PasswordsMatch,
    AlreadyExistingUser,
}

/// Create User from signup
/// Use the cookies and the App state
/// Get name, email, password and password confirmation
/// Return user_id
pub async fn create_user(
    state: &AppState,
    name: &String,
    email: &String,
    password: &String,
    confirm_password: &String,
) -> Result<String, UserError> {
    // Check if missing data
    if name.is_empty()
        || email.is_empty()
        || password.is_empty()
        || confirm_password.is_empty()
    {
        return Err(UserError::MissingInformation);
    }

    // Check if password does not match confirmation
    if password != confirm_password
    {
        return Err(UserError::PasswordsMatch);
    }

    // Encrypt the password
    let encrypted_password = encrypt_text(password).map_err(|_| UserError::CryptoError)?;

    // Insert the user
    let row = sqlx::query(
        "INSERT INTO users (name, email, encrypted_password) VALUES ($1, $2, $3) RETURNING user_id",
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

    // Return user_id
    Ok(row.get::<Uuid, &str>("user_id").to_string())
}
