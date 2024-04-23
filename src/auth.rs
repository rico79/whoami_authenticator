pub mod signin;
pub mod signout;
pub mod signup;

use std::fmt::Debug;
use std::fmt::Display;

use askama_axum::IntoResponse;
use axum::response::Redirect;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, Row};

use crate::utils::crypto::verify_encrypted_text;
use crate::AppState;

/// Error types for auth errors
#[derive(Debug, Deserialize)]
pub enum AuthError {
    DatabaseError,
    CryptoError,
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}

/// Remove session id from cookies
/// Then redirect to an url
pub fn remove_session_and_redirect(cookies: CookieJar, redirect_to: &str) -> impl IntoResponse {
    (
        cookies.remove(Cookie::from("session_id")),
        Redirect::to(redirect_to),
    )
}

/// Create session id in cookies if user credentials are Ok
/// Then redirect to an url
/// Use the cookies and the App state
/// Get email and password and the url for redirect
/// Return session id wich is a JWT or an AuthError
pub async fn create_session_from_credentials_and_redirect(
    cookies: CookieJar,
    state: &AppState,
    email: &String,
    password: &String,
    redirect_to: &str,
) -> Result<impl IntoResponse, AuthError> {
    // Check if missing credentials
    if email.is_empty() || password.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

    // Select the user with this email
    let query_result =
        sqlx::query("SELECT user_id, name, encrypted_password FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&state.db_pool)
            .await
            .map_err(|_| AuthError::DatabaseError)?;

    // Check if there is a user selected
    if let Some(row) = query_result {
        // Get the user data
        let user_id = row.get::<Uuid, &str>("user_id");
        let user_name = row.get::<String, &str>("name");
        let encrypted_password = row.get::<String, &str>("encrypted_password");

        // Check password
        if verify_encrypted_text(password, &encrypted_password)
            .map_err(|_| AuthError::CryptoError)?
        {
            // Generate and return JWT
            let jwt = generate_encoded_jwt(
                user_id.to_string(),
                user_name,
                email.to_string(),
                120,
                state.jwt_secret.clone(),
            )
            .map_err(|_| AuthError::TokenCreation)?;

            // Return Redirect with cookie containing the session_id
            Ok((
                cookies.add(Cookie::new("session_id", jwt)),
                Redirect::to(redirect_to),
            ))
        }
        // Wrong Password
        else {
            Err(AuthError::WrongCredentials)
        }
    }
    // No user found
    else {
        Err(AuthError::WrongCredentials)
    }
}

/// JWT claims struct
/// sub = subject -> user connected email
/// iss = issuer -> company name of the auth server
/// iat = issued at -> date of the token generation
/// exp = expiration -> end date of the token
#[derive(Debug, Serialize, Deserialize)]
pub struct JWTClaims {
    pub sub: String,
    pub name: String,
    pub email: String,
    iss: String,
    iat: i64,
    exp: i64,
}

/// Allow us to print the claim details
impl Display for JWTClaims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "User ID: {} - Name: {} - Email: {} - Issuing company: {}",
            self.sub, self.name, self.email, self.iss
        )
    }
}

/// Implement FromRequestParts for Claims (the JWT struct)
/// FromRequestParts allows us to use JWTClaims without consuming the request
#[async_trait]
impl<S> FromRequestParts<S> for JWTClaims
where
    AppState: FromRef<S>,
    S: Send + Sync + Debug,
{
    type Rejection = signin::PageTemplate;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract cookies
        let jar = parts
            .extract::<CookieJar>()
            .await
            .map_err(|_| signin::PageTemplate::from(None, Some(AuthError::InvalidToken)))?;

        // Extract token
        if let Some(token) = jar.get("session_id") {
            // Extract app state to get the jwt secret
            let state = parts
                .extract_with_state::<AppState, _>(state)
                .await
                .map_err(|_| signin::PageTemplate::from(None, Some(AuthError::InvalidToken)))?;

            // Decode the user data
            let token_data = decode::<JWTClaims>(
                token.value(),
                &DecodingKey::from_secret(state.jwt_secret.as_ref()),
                &Validation::default(),
            )
            .map_err(|_| signin::PageTemplate::from(None, Some(AuthError::InvalidToken)))?;

            Ok(token_data.claims)
        } else {
            Err(signin::PageTemplate::from(
                None,
                Some(AuthError::InvalidToken),
            ))
        }
    }
}

/// Generate an encoded JSON Web Token
/// The subject to pass in argument is for example the mail of the authenticated user
/// The token will expire after the nb of seconds passed in argument
fn generate_encoded_jwt(
    subject: String,
    user_name: String,
    user_email: String,
    seconds_before_expiration: i64,
    secret: String,
) -> jsonwebtoken::errors::Result<String> {
    // Generate issued at and expiration dates (X seconds after)
    let issued_at = Utc::now().timestamp();
    let expiration = issued_at + seconds_before_expiration;

    // Create token claims
    let claims = JWTClaims {
        sub: subject,
        name: user_name,
        email: user_email,
        iss: String::from("Brouclean Softwares Authenticator"),
        iat: issued_at,
        exp: expiration,
    };

    // Encode
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}
