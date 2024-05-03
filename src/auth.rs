pub mod signin;
pub mod signout;
pub mod signup;

use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;

use askama_axum::IntoResponse;
use axum::extract::Request;
use axum::response::Redirect;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use jsonwebtoken::{
    decode, encode, errors::ErrorKind::ExpiredSignature, DecodingKey, EncodingKey, Header,
    Validation,
};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::Utc;
use sqlx::types::Uuid;
use tracing::log::error;

use crate::apps::App;
use crate::general::message::Level;
use crate::general::message::MessageBlock;
use crate::utils::crypto::verify_encrypted_text;
use crate::AppState;

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

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            AuthError::DatabaseError => "Un problème est survenu, veuillez réessayer plus tard",
            AuthError::CryptoError => "Un problème est survenu, veuillez réessayer plus tard",
            AuthError::UserNotExisting => "L'utilisateur est inconnu",
            AuthError::WrongCredentials => "Les données de connexion sont incorrectes",
            AuthError::MissingCredentials => "Veuillez remplir votre mail et votre mot de passe",
            AuthError::TokenCreationFailed => {
                "Un problème est survenu, veuillez réessayer plus tard"
            }
            AuthError::InvalidToken => "",
        };

        write!(f, "{}", message)
    }
}

/// sub = subject -> user unique id
/// iss = issuer -> company name of the auth server
/// iat = issued at -> date of the token generation
/// exp = expiration -> end date of the token
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IdTokenClaims {
    sub: String,
    iss: String,
    iat: i64,
    exp: i64,
    pub name: String,
    pub mail: String,
}

impl IdTokenClaims {
    pub fn user_id(&self) -> Uuid {
        Uuid::parse_str(&self.sub).unwrap()
    }

    pub fn new(
        state: &AppState,
        user_id: Uuid,
        user_name: String,
        user_mail: String,
        seconds_before_expiration: i32,
    ) -> Self {
        let now = Utc::now().timestamp();

        let expiration_time = now + i64::from(seconds_before_expiration);

        IdTokenClaims {
            sub: user_id.to_string(),
            name: user_name,
            mail: user_mail,
            iss: state.authenticator_app.base_url.clone(),
            iat: now,
            exp: expiration_time,
        }
    }

    pub fn get_from_cookies(state: &AppState, cookies: &CookieJar) -> Result<Self, AuthError> {
        let token = cookies.get("session_id").ok_or(AuthError::InvalidToken)?;

        Self::decode(
            token.value().to_string(),
            state.authenticator_app.jwt_secret.clone(),
        )
    }

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

    pub fn decode(token: String, secret: String) -> Result<Self, AuthError> {
        let decoded_token = decode::<IdTokenClaims>(
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

        Ok(decoded_token.claims)
    }
}

impl Display for IdTokenClaims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "User Id: {} - Name: {} - Mail: {} - Issuing company: {}",
            self.sub, self.name, self.mail, self.iss
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
    type Rejection = signin::SigninPage;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = parts
            .extract_with_state::<AppState, _>(state)
            .await
            .unwrap();

        let request_uri = Request::from_parts(parts.clone(), state.clone())
            .uri()
            .clone();

        let cookie_jar = parts.extract::<CookieJar>().await.map_err(|error| {
            error!("{:?}", error);
            signin::SigninPage::from(
                &state,
                None,
                None,
                Some(request_uri.to_string()),
                MessageBlock::closeable(
                    Level::Error,
                    "",
                    &AuthError::InvalidToken.to_string(),
                ),
            )
        })?;

        Self::get_from_cookies(&state, &cookie_jar).map_err(|error| {
            signin::SigninPage::from(
                &state,
                None,
                None,
                Some(request_uri.to_string()),
                MessageBlock::closeable(Level::Error, "", &error.to_string()),
            )
        })
    }
}

pub fn remove_session_and_redirect(cookies: CookieJar, redirect_to: &str) -> impl IntoResponse {
    (
        cookies.remove(Cookie::from("session_id")),
        Redirect::to(redirect_to),
    )
}

pub fn create_session_into_response(
    cookies: CookieJar,
    jwt: String,
    response: impl IntoResponse,
) -> impl IntoResponse {
    (cookies.add(Cookie::new("session_id", jwt)), response)
}

pub async fn create_session_from_credentials_and_redirect_response(
    cookies: CookieJar,
    state: &AppState,
    mail: &String,
    password: &String,
    app_id: i32,
    redirect_to: Option<String>,
) -> Result<impl IntoResponse, AuthError> {
    if mail.is_empty() || password.is_empty() {
        return Err(AuthError::MissingCredentials);
    }

    let (user_id, user_name, encrypted_password): (Uuid, String, String) =
        sqlx::query_as("SELECT id, name, encrypted_password FROM users WHERE mail = $1")
            .bind(mail)
            .fetch_optional(&state.db_pool)
            .await
            .map_err(|error| {
                error!("{:?}", error);
                AuthError::DatabaseError
            })?
            .ok_or(AuthError::UserNotExisting)?;

    let password_is_not_ok =
        !verify_encrypted_text(password, &encrypted_password).map_err(|error| {
            error!("{:?}", error);
            AuthError::CryptoError
        })?;

    if password_is_not_ok {
        return Err(AuthError::WrongCredentials);
    }

    let id_token_claims = IdTokenClaims::new(
        state,
        user_id,
        user_name,
        mail.to_string(),
        state.authenticator_app.jwt_seconds_to_expire.clone(),
    );

    let id_token = id_token_claims.encode(state.authenticator_app.jwt_secret.clone())?;

    let redirect_response = App::select_app_or_authenticator(&state, app_id)
        .await
        .redirect_to_another_endpoint(redirect_to);

    let redirect_with_id_session_cookie =
        create_session_into_response(cookies, id_token, redirect_response);

    Ok(redirect_with_id_session_cookie)
}
