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
    auth::{extract_id_token_claims_from_session, signin},
    general::{
        message::{Level, MessageBlock},
        AuthenticatorError,
    },
    users::User,
    AppState,
};

pub struct JsonWebToken {
    authenticator_app: App,
    app: App,
}

impl JsonWebToken {
    pub fn for_app(state: &AppState, app: &App) -> Self {
        Self {
            authenticator_app: state.authenticator_app.clone(),
            app: app.clone(),
        }
    }

    pub fn for_authenticator(state: &AppState) -> Self {
        Self::for_app(state, &state.authenticator_app)
    }

    pub fn generate_id_token(
        &self,
        user: &User,
    ) -> Result<(String, IdTokenClaims), AuthenticatorError> {
        let now = Utc::now().timestamp();

        let expiration_time = now + i64::from(self.app.jwt_seconds_to_expire);

        let claims = IdTokenClaims {
            sub: user.id.to_string(),
            name: user.name.clone(),
            mail: user.mail.clone(),
            iss: self.authenticator_app.base_url.clone(),
            iat: now,
            exp: expiration_time,
        };

        let generated_token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.app.jwt_secret.as_ref()),
        )
        .map_err(|error| {
            error!("{:?}", error);
            AuthenticatorError::TokenCreationFailed
        })?;

        Ok((generated_token, claims))
    }

    pub fn extract_id_token(&self, token: String) -> Result<IdTokenClaims, AuthenticatorError> {
        let decoded_token = decode::<IdTokenClaims>(
            &token,
            &DecodingKey::from_secret(self.app.jwt_secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|error| {
            match error.kind() {
                ExpiredSignature => (),
                _ => error!("{:?}", error),
            };
            AuthenticatorError::InvalidToken
        })?;

        Ok(decoded_token.claims)
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
                MessageBlock::closeable(
                    Level::Error,
                    "",
                    &AuthenticatorError::InvalidToken.to_string(),
                ),
            )
        })?;

        extract_id_token_claims_from_session(&state, &cookie_jar, &state.authenticator_app).map_err(
            |error| {
                signin::SigninPage::for_app_with_redirect_and_message(
                    state.authenticator_app.clone(),
                    Some(request_uri.to_string()),
                    MessageBlock::closeable(Level::Error, "", &error.to_string()),
                )
            },
        )
    }
}
