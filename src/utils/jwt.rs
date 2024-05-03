use core::fmt::Debug;
use std::fmt::Display;

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, Request},
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::extract::CookieJar;
use jsonwebtoken::{
    decode, encode, errors::ErrorKind::ExpiredSignature, DecodingKey, EncodingKey, Header,
    Validation,
};
use serde::{Deserialize, Serialize};
use sqlx::types::{chrono::Utc, Uuid};
use tracing::error;

use crate::{
    apps::App,
    auth::{signin, AuthError},
    general::message::{Level, MessageBlock},
    users::User,
    AppState,
};

pub enum JWT {
    IdToken,
}

impl Display for JWT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JWT::IdToken => write!(f, "id_token"),
        }
    }
}

pub struct JWTGenerator {
    authenticator_app: App,
    app: App,
    user: User,
}

impl JWTGenerator {
    pub fn new(state: &AppState, app: &App, user: &User) -> Self {
        Self {
            authenticator_app: state.authenticator_app.clone(),
            app: app.clone(),
            user: user.clone(),
        }
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
        seconds_to_expire: i32,
    ) -> Self {
        let now = Utc::now().timestamp();

        let expiration_time = now + i64::from(seconds_to_expire);

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
            signin::SigninPage::for_app_with_redirect_and_message(
                state.authenticator_app.clone(),
                Some(request_uri.to_string()),
                MessageBlock::closeable(Level::Error, "", &AuthError::InvalidToken.to_string()),
            )
        })?;

        Self::get_from_cookies(&state, &cookie_jar).map_err(|error| {
            signin::SigninPage::for_app_with_redirect_and_message(
                state.authenticator_app.clone(),
                Some(request_uri.to_string()),
                MessageBlock::closeable(Level::Error, "", &error.to_string()),
            )
        })
    }
}
