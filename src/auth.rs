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
use sqlx::types::Uuid;
use tracing::log::error;

use crate::apps::App;
use crate::utils::crypto::verify_encrypted_text;
use crate::utils::jwt::IdTokenClaims;
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

pub fn remove_session_and_redirect(cookies: CookieJar, redirect_to: &str) -> impl IntoResponse {
    (
        cookies.remove(Cookie::from("session_id")),
        Redirect::to(redirect_to),
    )
}

pub async fn create_session_into_response(
    cookies: CookieJar,
    state: &AppState,
    user_id: Uuid,
    user_name: String,
    user_mail: String,
    app_to_connect: &App,
    requested_endpoint: Option<String>,
) -> Result<impl IntoResponse, AuthError> {
    let claims = IdTokenClaims::new(
        state,
        user_id,
        user_name,
        user_mail,
        app_to_connect.jwt_seconds_to_expire,
    );

    let id_token = claims.encode(app_to_connect.jwt_secret.clone())?;

    let redirect = app_to_connect
        .redirect_to_endpoint(requested_endpoint)
        .clone();

    let response_with_session_cookie = (cookies.add(Cookie::new("session_id", id_token)), redirect);

    Ok(response_with_session_cookie)
}

pub async fn create_session_from_credentials_and_redirect_response(
    cookies: CookieJar,
    state: &AppState,
    mail: &String,
    password: &String,
    app_id: i32,
    requested_endpoint: Option<String>,
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

    let app_to_connect = App::select_app_or_authenticator(&state, app_id).await;

    create_session_into_response(
        cookies,
        state,
        user_id,
        user_name,
        mail.to_string(),
        &app_to_connect,
        requested_endpoint,
    )
    .await
}
