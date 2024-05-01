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
use jsonwebtoken::{
    decode, encode, errors::ErrorKind::ExpiredSignature, DecodingKey, EncodingKey, Header,
    Validation,
};
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, Row};
use tracing::log::error;

use crate::apps::App;
use crate::utils::crypto::verify_encrypted_text;
use crate::AppState;

/// Error types for auth errors
#[derive(Debug, Deserialize)]
pub enum AuthError {
    DatabaseError,
    CryptoError,
    UserNotExisting,
    WrongCredentials,
    MissingCredentials,
    TokenCreationFailed,
    InvalidToken,
}

/// IdTokenClaims struct
/// sub = subject -> user unique id
/// iss = issuer -> company name of the auth server
/// iat = issued at -> date of the token generation
/// exp = expiration -> end date of the token
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdTokenClaims {
    sub: String,
    pub name: String,
    pub email: String,
    iss: String,
    iat: i64,
    exp: i64,
}

impl IdTokenClaims {
    /// Get user id
    pub fn user_id(&self) -> Uuid {
        Uuid::parse_str(&self.sub).unwrap()
    }

    /// New IdTokenClaims based on user data
    /// The token will expire after the nb of seconds passed in argument
    pub fn new(
        user_id: Uuid,
        user_name: String,
        user_email: String,
        seconds_before_expiration: i32,
    ) -> Self {
        // Generate issued at and expiration dates (X seconds after)
        let issued_at = Utc::now().timestamp();
        let expiration = issued_at + i64::from(seconds_before_expiration);

        // Create token claims
        IdTokenClaims {
            sub: user_id.to_string(),
            name: user_name,
            email: user_email,
            iss: String::from("Brouclean Softwares Authenticator"),
            iat: issued_at,
            exp: expiration,
        }
    }

    /// Get claims from cookies
    /// Get the app state and cookie jar
    /// return claims
    pub fn get_from_cookies(state: &AppState, cookies: &CookieJar) -> Result<Self, AuthError> {
        // Extract token
        if let Some(token) = cookies.get("session_id") {
            Self::decode(
                token.value().to_string(),
                state.authenticator_app.jwt_secret.clone(),
            )
        } else {
            Err(AuthError::InvalidToken)
        }
    }

    /// Generate an encoded JSON Web Token
    pub fn encode(&self, secret: String) -> Result<String, AuthError> {
        encode(
            &Header::default(),
            self,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|error| {
            error!("{:?}", error);
            AuthError::TokenCreationFailed
        })
    }

    /// Decode JSON Web Token
    pub fn decode(token: String, secret: String) -> Result<Self, AuthError> {
        // Decode the user data
        let token_data = decode::<IdTokenClaims>(
            &token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|error| {
            match error.kind() {
                ExpiredSignature => (),
                _ => error!("{:?}", error),
            };
            AuthError::InvalidToken
        })?;

        Ok(token_data.claims)
    }
}

/// Allow us to print the claim details
impl Display for IdTokenClaims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "User Id: {} - Name: {} - Email: {} - Issuing company: {}",
            self.sub, self.name, self.email, self.iss
        )
    }
}

/// Implement FromRequestParts for Claims (the JWT struct)
/// FromRequestParts allows us to use JWTClaims without consuming the request
#[async_trait]
impl<S> FromRequestParts<S> for IdTokenClaims
where
    AppState: FromRef<S>,
    S: Send + Sync + Debug,
{
    type Rejection = signin::PageTemplate;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Extract app state to get the jwt secret
        let state = parts
            .extract_with_state::<AppState, _>(state)
            .await
            .unwrap();

        // Extract cookies
        let cookies = parts.extract::<CookieJar>().await.map_err(|error| {
            error!("{:?}", error);
            signin::PageTemplate::from(&state, None, None, Some(AuthError::InvalidToken))
        })?;

        // Extract token
        Self::get_from_cookies(&state, &cookies)
            .map_err(|error| signin::PageTemplate::from(&state, None, None, Some(error)))
    }
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
/// Use the cookies and jwt
/// Return response
pub fn create_session_into_response(
    cookies: CookieJar,
    jwt: String,
    response: impl IntoResponse,
) -> impl IntoResponse {
    (cookies.add(Cookie::new("session_id", jwt)), response)
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
    app_id: i32,
) -> Result<impl IntoResponse, AuthError> {
    // Check if missing credentials
    if email.is_empty() || password.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

    // Select the user with this email
    let query_result =
        sqlx::query("SELECT id, name, encrypted_password FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&state.db_pool)
            .await
            .map_err(|error| {
                error!("{:?}", error);
                AuthError::DatabaseError
            })?;

    // Check if there is a user selected
    if let Some(row) = query_result {
        // Get the user data
        let user_id = row.get::<Uuid, &str>("id");
        let user_name = row.get::<String, &str>("name");
        let encrypted_password = row.get::<String, &str>("encrypted_password");

        // Check password
        if verify_encrypted_text(password, &encrypted_password).map_err(|error| {
            error!("{:?}", error);
            AuthError::CryptoError
        })? {
            // Generate JWT
            let jwt = IdTokenClaims::new(
                user_id,
                user_name,
                email.to_string(),
                state.authenticator_app.jwt_seconds_to_expire.clone(),
            )
            .encode(state.authenticator_app.jwt_secret.clone())?;

            // Return Redirect with cookie containing the session_id
            Ok(create_session_into_response(
                cookies,
                jwt,
                App::select_app_or_authenticator(&state, app_id)
                    .await
                    .redirect_to(),
            ))
        }
        // Wrong Password
        else {
            Err(AuthError::WrongCredentials)
        }
    }
    // No user found
    else {
        Err(AuthError::UserNotExisting)
    }
}
