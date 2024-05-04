pub mod signin;
pub mod signout;
pub mod signup;

use askama_axum::IntoResponse;
use axum::response::Redirect;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use time::Duration;

use crate::apps::App;
use crate::general::AuthenticatorError;
use crate::users::User;
use crate::utils::jwt::{IdClaims, TokenFactory};
use crate::AppState;

const SESSION_TOKEN: &str = "session_token";

pub fn remove_session_and_redirect(cookies: CookieJar, redirect_to: &str) -> impl IntoResponse {
    (
        cookies.remove(Cookie::build(SESSION_TOKEN).path("/")),
        Redirect::to(redirect_to),
    )
}

pub fn extract_session_claims(
    state: &AppState,
    cookies: &CookieJar,
    app: &App,
) -> Result<IdClaims, AuthenticatorError> {
    let token = cookies
        .get(SESSION_TOKEN)
        .ok_or(AuthenticatorError::InvalidToken)?;

    let token = TokenFactory::for_app(state, app).extract_id_token(token.value().to_string())?;

    Ok(token.claims)
}

pub fn new_session_into_response(
    cookies: CookieJar,
    state: &AppState,
    user: &User,
    app_to_connect: &App,
    requested_endpoint: Option<String>,
) -> Result<impl IntoResponse, AuthenticatorError> {
    let session_duration = app_to_connect.jwt_seconds_to_expire.clone();

    let id_token = TokenFactory::for_app(state, app_to_connect).generate_id_token(user)?;

    let redirect = app_to_connect
        .redirect_to_endpoint(requested_endpoint)
        .clone();

    let secure_domain = app_to_connect.domain()?;

    let cookie = Cookie::build((SESSION_TOKEN, id_token.token))
        .domain(secure_domain)
        .path("/")
        .secure(true)
        .http_only(true)
        .max_age(Duration::seconds(session_duration.into()));

    let response_with_session_cookie = (cookies.add(cookie), redirect);

    Ok(response_with_session_cookie)
}
