pub mod signin;
pub mod signout;
pub mod signup;

use std::fmt::Debug;
use std::fmt::Display;

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

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

/// JWT claims struct
/// sub = subject -> user connected email
/// iss = issuer -> company name of the auth server
/// iat = issued at -> date of the token generation
/// exp = expiration -> end date of the token
#[derive(Debug, Serialize, Deserialize)]
struct JWTClaims {
    sub: String,
    iss: String,
    iat: i64,
    exp: i64,
}

/// Allow us to print the claim details
impl Display for JWTClaims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User ID: {}\nIssuing company: {}", self.sub, self.iss)
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
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| signin::PageTemplate::from(None, Some(AuthError::InvalidToken)))?;

        // Extract app state to get the jwt secret
        let state = parts
            .extract_with_state::<AppState, _>(state)
            .await
            .map_err(|_| signin::PageTemplate::from(None, Some(AuthError::InvalidToken)))?;

        // Decode the user data
        let token_data = decode::<JWTClaims>(
            bearer.token(),
            &DecodingKey::from_secret(state.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| signin::PageTemplate::from(None, Some(AuthError::InvalidToken)))?;

        Ok(token_data.claims)
    }
}

/// Generate an encoded JSON Web Token
/// The subject to pass in argument is for example the mail of the authenticated user
/// The token will expire after the nb of seconds passed in argument
pub fn generate_encoded_jwt(
    subject: &str,
    seconds_before_expiration: i64,
    secret: String,
) -> jsonwebtoken::errors::Result<String> {
    // Generate issued at and expiration dates (X seconds after)
    let issued_at = Utc::now().timestamp();
    let expiration = issued_at + seconds_before_expiration;

    // Create token claims
    let claims = JWTClaims {
        sub: subject.to_string(),
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