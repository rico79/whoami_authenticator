pub mod signin;
pub mod signout;
pub mod signup;

use askama_axum::IntoResponse;
use axum::extract::{FromRef, FromRequestParts, Request};
use axum::response::Redirect;
use axum::{async_trait, RequestPartsExt};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use core::fmt::Debug;
use http::request::Parts;
use sqlx::types::Uuid;
use time::{Date, Duration, OffsetDateTime};
use tracing::log::error;

use crate::apps::App;
use crate::general::message::{Level, MessageBlock};
use crate::general::AuthenticatorError;
use crate::users::User;
use crate::utils::jwt::TokenFactory;
use crate::AppState;

const SESSION_TOKEN: &str = "session_token";

#[derive(Clone, Debug)]
pub struct IdSession {
    pub user_id: Uuid,
    pub name: String,
    pub mail: String,
    pub avatar: String,
    pub birthday: Date,
    pub seconds_to_expire: i64,
}

#[async_trait]
impl<S> FromRequestParts<S> for IdSession
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
                MessageBlock::new(
                    Level::Error,
                    "",
                    &AuthenticatorError::InvalidToken.to_string(),
                ),
            )
        })?;

        let id_session = Self::extract(state.clone(), cookie_jar).map_err(|error| {
            signin::SigninPage::for_app_with_redirect_and_message(
                state.authenticator_app.clone(),
                Some(request_uri.to_string()),
                MessageBlock::new(Level::Error, "", &error.to_string()),
            )
        })?;

        Ok(id_session)
    }
}

impl IdSession {
    pub fn remove_and_redirect_to(cookies: CookieJar, redirect_to: &str) -> impl IntoResponse {
        (
            cookies
                .clone()
                .remove(Cookie::build(SESSION_TOKEN).path("/")),
            Redirect::to(redirect_to),
        )
    }

    fn extract(state: AppState, cookies: CookieJar) -> Result<Self, AuthenticatorError> {
        let token = cookies
            .get(SESSION_TOKEN)
            .ok_or(AuthenticatorError::InvalidToken)?;

        let id_claims = TokenFactory::for_authenticator(&state)
            .extract_id_token(token.value().to_string())?
            .claims;

        let now = OffsetDateTime::now_utc().unix_timestamp();

        Ok(IdSession {
            user_id: id_claims.user_id(),
            name: id_claims.name,
            mail: id_claims.mail,
            avatar: id_claims.avatar,
            birthday: id_claims.birthday,
            seconds_to_expire: id_claims.exp - now,
        })
    }

    pub fn set_with_redirect_to_app_endpoint(
        cookies: CookieJar,
        state: &AppState,
        user: &User,
        app_to_redirect: &App,
        requested_endpoint: Option<String>,
    ) -> Result<impl IntoResponse, AuthenticatorError> {
        let session_duration = state.authenticator_app.jwt_seconds_to_expire.clone();

        let id_token = TokenFactory::for_authenticator(state).generate_id_token(user)?;

        let secure_domain = state.authenticator_app.domain()?;

        let cookie = Cookie::build((SESSION_TOKEN, id_token.token))
            .domain(secure_domain)
            .path("/")
            .secure(true)
            .http_only(true)
            .max_age(Duration::seconds(session_duration.into()));

        let redirect = app_to_redirect
            .redirect_to_endpoint(requested_endpoint)
            .clone();

        let response_with_session_cookie = (cookies.add(cookie), redirect);

        Ok(response_with_session_cookie)
    }
}
