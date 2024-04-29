pub mod app_list;

use axum::response::Redirect;
use serde::Deserialize;
use sqlx::{
    postgres::PgRow,
    types::{time::OffsetDateTime, Uuid},
    FromRow, Row,
};
use tracing::log::error;

use crate::{auth::IdTokenClaims, utils::date_time::DateTime, AppState};

/// Error types
#[derive(Debug, Deserialize)]
pub enum AppError {
    DatabaseError,
    InvalidId,
    NotFound,
}

/// App struct
#[derive(Clone, Debug)]
pub struct App {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_url: String,
    redirect_endpoint: String,
    logo_endpoint: String,
    pub jwt_secret: String,
    pub jwt_seconds_to_expire: i64,
    pub created_at: DateTime,
    pub owner_email: Option<String>,
}

/// To get App from database
impl FromRow<'_, PgRow> for App {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get::<Uuid, &str>("app_id")?.to_string(),
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            base_url: row.try_get("base_url")?,
            redirect_endpoint: row.try_get("redirect_endpoint")?,
            logo_endpoint: row.try_get("logo_endpoint")?,
            jwt_secret: row.try_get("jwt_secret")?,
            jwt_seconds_to_expire: row.try_get("jwt_seconds_to_expire")?,
            created_at: DateTime::from(row.try_get::<OffsetDateTime, &str>("created_at")?),
            owner_email: row.try_get("owner_email")?,
        })
    }
}

/// Authenticator is the default app
impl Default for App {
    fn default() -> Self {
        Self::init_authenticator_app(
            "".to_owned(),
            "".to_owned(),
            0,
            DateTime::default(),
            None,
        )
    }
}

impl App {
    /// Authenticator app
    pub fn init_authenticator_app(
        base_url: String,
        jwt_secret: String,
        jwt_seconds_to_expire: i64,
        created_at: DateTime,
        owner_email: Option<String>,
    ) -> Self {
        Self {
            id: "".to_owned(),
            name: "Authenticator".to_owned(),
            description: "Gère la connexion de vos utilisateurs pour vos apps".to_owned(),
            base_url,
            redirect_endpoint: "/welcome".to_owned(),
            logo_endpoint: "/assets/images/logo.png".to_owned(),
            jwt_secret,
            jwt_seconds_to_expire,
            created_at,
            owner_email,
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

    /// Create redirect url
    pub fn redirect_url(&self) -> String {
        format!("{}{}", &self.base_url, &self.redirect_endpoint)
    }

    /// Create logo url
    pub fn logo_url(&self) -> String {
        format!("{}{}", &self.base_url, &self.logo_endpoint)
    }

    /// App redirection
    /// Redirect to the app after signin
    pub fn redirect_to(&self) -> Redirect {
        Redirect::to(&self.redirect_url())
    }

    /// Select all apps owned by the user
    /// Get user_id
    /// return list of apps
    pub async fn select_own_apps(
        state: &AppState,
        claims: &IdTokenClaims,
    ) -> Result<Vec<Self>, AppError> {
        // Convert the user id into Uuid
        let user_uuid = Uuid::parse_str(&claims.sub).map_err(|error| {
            error!("{:?}", error);
            AppError::InvalidId
        })?;

        // Get apps from database
        let mut apps: Vec<App> = sqlx::query_as(
            "SELECT 
                a.app_id, 
                a.name, 
                a.description, 
                a.base_url, 
                a.redirect_endpoint, 
                a.logo_endpoint, 
                a.jwt_secret, 
                a.jwt_seconds_to_expire, 
                a.created_at, 
                u.email as owner_email
            FROM 
                users u 
            JOIN 
                apps a ON u.id = a.owner_id 
            WHERE 
                u.user_id = $1",
        )
        .bind(user_uuid)
        .fetch_all(&state.db_pool)
        .await
        .map_err(|error| {
            error!("{:?}", error);
            AppError::DatabaseError
        })?;

        // Add Authenticator if same email than this app mailer user
        if state.authenticator_app.owner_email.clone().unwrap_or_default() == claims.email {
            apps.push(state.authenticator_app.clone())
        }

        Ok(apps)
    }

    /// Select app
    /// Get app_id
    /// return app
    pub async fn select_from_app_id(state: &AppState, app_id: &String) -> Result<Self, AppError> {
        // Convert the app id into Uuid
        let app_uuid = Uuid::parse_str(app_id).map_err(|error| {
            error!("{:?}", error);
            AppError::InvalidId
        })?;

        // Get app from database
        let app: App = sqlx::query_as(
            "SELECT 
                a.app_id, 
                a.name, 
                a.description, 
                a.base_url, 
                a.redirect_endpoint, 
                a.logo_endpoint, 
                a.jwt_secret, 
                a.jwt_seconds_to_expire, 
                a.created_at, 
                u.email as owner_email
            FROM apps a 
            LEFT OUTER JOIN users u ON 
                u.id = a.owner_id 
            WHERE 
                a.app_id = $1",
        )
        .bind(app_uuid)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|error| {
            error!("{:?}", error);
            AppError::NotFound
        })?;

        Ok(app)
    }
}
