pub mod signin;
pub mod signout;
pub mod signup;

use std::fmt;
use std::fmt::Debug;

use askama_axum::IntoResponse;
use axum::response::Redirect;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::apps::App;
use crate::users::User;
use crate::utils::jwt::JWTGenerator;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub enum AuthError {
    DatabaseError,
    CryptoError,
    NotExistingUser,
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
            AuthError::NotExistingUser => "L'utilisateur est inconnu",
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
) -> Result<impl IntoResponse, AuthError> {
    let (id_token, _) = JWTGenerator::new(state, app_to_connect, user).generate_id_token()?;

    let redirect = app_to_connect
        .redirect_to_endpoint(requested_endpoint)
        .clone();

    let response_with_session_cookie = (cookies.add(Cookie::new("session_id", id_token)), redirect);

    Ok(response_with_session_cookie)
}
