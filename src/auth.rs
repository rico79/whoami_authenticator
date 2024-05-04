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
use crate::utils::jwt::{IdClaims, TokenFactory};
use crate::AppState;

pub fn remove_id_token_and_redirect(cookies: CookieJar, redirect_to: &str) -> impl IntoResponse {
    (
        cookies.remove(Cookie::from("id_token")),
        Redirect::to(redirect_to),
    )
}

pub fn extract_id_token_claims(
    state: &AppState,
    cookies: &CookieJar,
    app: &App,
) -> Result<IdClaims, AuthenticatorError> {
    let token = cookies
        .get("id_token")
        .ok_or(AuthenticatorError::InvalidToken)?;

    let token = TokenFactory::for_app(state, app).extract_id_token(token.value().to_string())?;

    Ok(token.claims)
}

pub fn put_new_id_token_into_response(
    cookies: CookieJar,
    state: &AppState,
    user: &User,
    app_to_connect: &App,
    requested_endpoint: Option<String>,
) -> Result<impl IntoResponse, AuthenticatorError> {
    let id_token = TokenFactory::for_app(state, app_to_connect)
        .generate_id_token(user)?
        .token;

    let redirect = app_to_connect
        .redirect_to_endpoint(requested_endpoint)
        .clone();

    let response_with_session_cookie = (cookies.add(Cookie::new("id_token", id_token)), redirect);

    Ok(response_with_session_cookie)
}
