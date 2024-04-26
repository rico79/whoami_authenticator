use axum::response::Redirect;
use serde::Deserialize;

use crate::AppState;

/// Error types
#[derive(Debug, Deserialize)]
pub enum AppError {
    DatabaseError,
}

/// App struct
#[derive(Clone, Debug)]
pub struct App {
    pub id: String,
    pub name: String,
    pub base_url: String,
    welcome_endpoint: String,
    logo_endpoint: String,
    pub jwt_secret: String,
    pub jwt_seconds_to_expire: i64,
}

/// Authenticator is the default app
impl Default for App {
    fn default() -> Self {
        Self::init_authenticator_app("".to_owned(), "".to_owned(), 0)
    }
}

impl App {
    /// Authenticator app
    pub fn init_authenticator_app(
        base_url: String,
        jwt_secret: String,
        jwt_seconds_to_expire: i64,
    ) -> Self {
        Self {
            id: "".to_owned(),
            name: "Authenticator".to_owned(),
            base_url,
            welcome_endpoint: "/welcome".to_owned(),
            logo_endpoint: "/assets/images/logo.png".to_owned(),
            jwt_secret,
            jwt_seconds_to_expire,
        }
    }

    /// Get app from id
    /// If no app is found return authenticator app
    pub fn select_app_or_authenticator(state: &AppState, app_id: &String) -> Self {
        if app_id.len() > 0 {
            // Get App

            // Return values
            state.authenticator_app.clone()
        } else {
            state.authenticator_app.clone()
        }
    }

    /// Create welcome url
    pub fn welcome_url(&self) -> String {
        format!("{}{}", &self.base_url, &self.welcome_endpoint)
    }

    /// Create logo url
    pub fn logo_url(&self) -> String {
        format!("{}{}", &self.base_url, &self.logo_endpoint)
    }

    /// App redirection
    /// Redirect to the app welcome page
    pub fn redirect_to_welcome(&self) -> Redirect {
        Redirect::to(&self.welcome_url())
    }
}
