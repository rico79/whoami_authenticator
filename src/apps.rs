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
    pub welcome_url: String,
    pub logo_url: String,
}

impl App {
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

    /// App redirection
    /// Redirect to the app welcome page
    pub fn redirect_to_welcome(&self) -> Redirect {
        Redirect::to(&self.welcome_url)
    }
}

/// Authenticator is the default app
impl Default for App {
    fn default() -> Self {
        Self {
            id: "".to_owned(),
            name: "Authenticator".to_owned(),
            welcome_url: "/welcome".to_owned(),
            logo_url: "assets/images/logo.png".to_owned(),
        }
    }
}
