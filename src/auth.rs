pub mod signin;
pub mod signout;
pub mod signup;

use askama_axum::IntoResponse;
use axum::response::Redirect;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;

use crate::apps::App;
use crate::general::AuthenticatorError;
use crate::users::User;
use crate::utils::jwt::{IdTokenClaims, JsonWebToken};
use crate::AppState;

pub fn remove_session_and_redirect(cookies: CookieJar, redirect_to: &str) -> impl IntoResponse {
    (
        cookies.remove(Cookie::from("session_id")),
        Redirect::to(redirect_to),
    )
}

pub fn extract_id_token_claims_from_session(
    state: &AppState,
    cookies: &CookieJar,
    app: &App,
) -> Result<IdTokenClaims, AuthenticatorError> {
    let token = cookies
        .get("session_id")
        .ok_or(AuthenticatorError::InvalidToken)?;

    JsonWebToken::for_app(state, app).extract_id_token(token.value().to_string())
}

pub async fn create_session_into_response(
    cookies: CookieJar,
    state: &AppState,
    user: &User,
    app_to_connect: &App,
    requested_endpoint: Option<String>,
) -> Result<impl IntoResponse, AuthenticatorError> {
    let (id_token, _) = JsonWebToken::for_app(state, app_to_connect).generate_id_token(user)?;

    let redirect = app_to_connect
        .redirect_to_endpoint(requested_endpoint)
        .clone();

    let response_with_session_cookie = (cookies.add(Cookie::new("session_id", id_token)), redirect);

    Ok(response_with_session_cookie)
}
