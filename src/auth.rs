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
use crate::utils::jwt::JWTGenerator;
use crate::AppState;

pub fn remove_session_and_redirect(cookies: CookieJar, redirect_to: &str) -> impl IntoResponse {
    (
        cookies.remove(Cookie::from("session_id")),
        Redirect::to(redirect_to),
    )
}

pub async fn create_session_into_response(
    cookies: CookieJar,
    state: &AppState,
    user: &User,
    app_to_connect: &App,
    requested_endpoint: Option<String>,
) -> Result<impl IntoResponse, AuthenticatorError> {
    let (id_token, _) = JWTGenerator::new(state, app_to_connect, user).generate_id_token()?;

    let redirect = app_to_connect
        .redirect_to_endpoint(requested_endpoint)
        .clone();

    let response_with_session_cookie = (cookies.add(Cookie::new("session_id", id_token)), redirect);

    Ok(response_with_session_cookie)
}
