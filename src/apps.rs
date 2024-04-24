use axum::response::Redirect;

/// App struct
#[derive(Clone, Debug)]
pub struct App {
    pub app_id: String,
    pub name: String,
    welcome_url: String,
    pub logo_url: String,
}

impl App {
    /// Get authenticator app
    pub fn authenticator_app() -> Self {
        App {
            app_id: "".to_owned(),
            name: "Authenticator".to_owned(),
            welcome_url: "/welcome".to_owned(),
            logo_url: "assets/images/logo.png".to_owned(),
        }
    }

    /// Get app from id
    pub fn from_app_id(app_id: String) -> Self {
        if app_id.len() > 0 {
            // Get App

            // Return values
            Self::authenticator_app()
        } else {
            Self::authenticator_app()
        }
    }

    /// App redirection
    /// Redirect to the app welcome page
    pub fn redirect_to_welcome(&self) -> Redirect {
        Redirect::to(&self.welcome_url)
    }
}
