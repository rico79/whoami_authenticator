pub mod app;
pub mod app_list;

use axum::response::Redirect;
use serde::Deserialize;
use sqlx::{
    postgres::PgRow,
    types::{time::OffsetDateTime, Uuid},
    FromRow, PgPool, Row,
};
use tracing::log::error;

use crate::{auth::IdTokenClaims, users::User, utils::date_time::DateTime, AppState};

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
    pub id: i32,
    pub name: String,
    pub description: String,
    pub base_url: String,
    redirect_endpoint: String,
    logo_endpoint: String,
    pub jwt_secret: String,
    pub jwt_seconds_to_expire: i32,
    pub created_at: DateTime,
    pub owner_id: Option<Uuid>,
}

/// To get App from database
impl FromRow<'_, PgRow> for App {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            base_url: row.try_get("base_url")?,
            redirect_endpoint: row.try_get("redirect_endpoint")?,
            logo_endpoint: row.try_get("logo_endpoint")?,
            jwt_secret: row.try_get("jwt_secret")?,
            jwt_seconds_to_expire: row.try_get("jwt_seconds_to_expire")?,
            created_at: DateTime::from(row.try_get::<OffsetDateTime, &str>("created_at")?),
            owner_id: row.try_get("owner_id")?,
        })
    }
}

impl App {
    /// New app
    pub fn new(owner_id: &Uuid) -> Self {
        Self {
            id: -1,
            name: "".to_owned(),
            description: "".to_owned(),
            base_url: "".to_owned(),
            redirect_endpoint: "".to_owned(),
            logo_endpoint: "".to_owned(),
            jwt_secret: "".to_owned(),
            jwt_seconds_to_expire: 0,
            created_at: DateTime::default(),
            owner_id: Some(owner_id.clone()),
        }
    }

    /// Check if new app
    pub fn is_new(&self) -> bool {
        self.id < 0
    }

    /// Authenticator app
    pub async fn init_authenticator_app(
        db_pool: &PgPool,
        base_url: String,
        jwt_secret: String,
        jwt_seconds_to_expire: i32,
        created_at: DateTime,
        email: String,
    ) -> Self {
        Self {
            id: 0,
            name: "Authenticator".to_owned(),
            description: "Gère la connexion de vos utilisateurs pour vos apps".to_owned(),
            base_url,
            redirect_endpoint: "/welcome".to_owned(),
            logo_endpoint: "/assets/images/logo.png".to_owned(),
            jwt_secret,
            jwt_seconds_to_expire,
            created_at,
            owner_id: User::select_from_email(&db_pool, &email)
                .await
                .map(|owner| owner.id)
                .ok(),
        }
    }

    /// Check if app is authenticator
    pub fn is_authenticator_app(&self) -> bool {
        self.id == 0
    }

    /// Check if this user email can update the app
    /// Can update if owner
    /// NOTE that authenticator app can not be updated
    pub fn can_be_updated_by(&self, user_id: Uuid) -> bool {
        !self.is_authenticator_app()
            && match self.owner_id.clone() {
                Some(owner_id) => owner_id == user_id.clone(),
                None => false,
            }
    }

    /// Get app from id
    /// If no app is found return authenticator app
    pub async fn select_app_or_authenticator(state: &AppState, app_id: i32) -> Self {
        Self::select_from_app_id(state, app_id)
            .await
            .unwrap_or(state.authenticator_app.clone())
    }

    /// Create redirect url
    pub fn redirect_url(&self) -> String {
        format!("{}{}", &self.base_url, &self.redirect_endpoint)
    }

    /// Create logo url
    pub fn logo_url(&self) -> String {
        if self.base_url.len() > 0 && self.logo_endpoint.len() > 0 {
            format!("{}{}", &self.base_url, &self.logo_endpoint)
        } else {
            "/assets/images/app.png".to_owned()
        }
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
        // Get apps from database
        let mut apps: Vec<App> = sqlx::query_as(
            "SELECT 
                id, 
                name, 
                description, 
                base_url, 
                redirect_endpoint, 
                logo_endpoint, 
                jwt_secret, 
                jwt_seconds_to_expire, 
                created_at, 
                owner_id
            FROM apps 
            WHERE 
                owner_id = $1",
        )
        .bind(claims.user_id())
        .fetch_all(&state.db_pool)
        .await
        .map_err(|error| {
            error!(
                "Selecting apps for owner {} -> {:?}",
                claims.user_id(),
                error
            );
            AppError::DatabaseError
        })?;

        // Add Authenticator if same email than this app mailer user
        if state.authenticator_app.owner_id.clone().unwrap_or_default() == claims.user_id() {
            apps.push(state.authenticator_app.clone())
        }

        Ok(apps)
    }

    /// Select app
    /// Get app_id
    /// return app
    pub async fn select_from_app_id(state: &AppState, app_id: i32) -> Result<Self, AppError> {
        // Check if authenticator
        if app_id == state.authenticator_app.id {
            return Ok(state.authenticator_app.clone());
        }

        // Get app from database
        let app: App = sqlx::query_as(
            "SELECT 
                id, 
                name, 
                description, 
                base_url, 
                redirect_endpoint, 
                logo_endpoint, 
                jwt_secret, 
                jwt_seconds_to_expire, 
                created_at, 
                owner_id
            FROM apps
            WHERE 
                id = $1",
        )
        .bind(app_id)
        .fetch_one(&state.db_pool)
        .await
        .map_err(|error| {
            error!("Selecting apps from id {} -> {:?}", app_id, error);
            AppError::NotFound
        })?;

        Ok(app)
    }

    /// Save app
    /// return app
    pub async fn save(&self, state: &AppState, claims: &IdTokenClaims) -> Result<Self, AppError> {
        // Check if authenticator
        if self.is_authenticator_app() {
            return Ok(state.authenticator_app.clone());
        }

        // Check if missing Data

        // Save the App
        if self.is_new() {
            // Insert if new
            let app: App = sqlx::query_as(
                "INSERT INTO apps (
                    name, 
                    description, 
                    base_url, 
                    redirect_endpoint, 
                    logo_endpoint, 
                    jwt_secret, 
                    jwt_seconds_to_expire, 
                    owner_id) 
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
                RETURNING 
                    id,
                    name, 
                    description, 
                    base_url, 
                    redirect_endpoint, 
                    logo_endpoint, 
                    jwt_secret, 
                    jwt_seconds_to_expire, 
                    created_at, 
                    owner_id",
            )
            .bind(self.name.clone())
            .bind(self.description.clone())
            .bind(self.base_url.clone())
            .bind(self.redirect_endpoint.clone())
            .bind(self.logo_endpoint.clone())
            .bind(self.jwt_secret.clone())
            .bind(self.jwt_seconds_to_expire.clone())
            .bind(claims.user_id())
            .fetch_one(&state.db_pool)
            .await
            .map_err(|error| {
                error!("Inserting app {:?} -> {:?}", self, error);
                AppError::NotFound
            })?;

            Ok(app)
        } else {
            // Update otherwise
            let app: App = sqlx::query_as(
                "UPDATE apps
                SET
                    name = $1, 
                    description = $2, 
                    base_url = $3, 
                    redirect_endpoint = $4, 
                    logo_endpoint = $5, 
                    jwt_secret = $6, 
                    jwt_seconds_to_expire = $7
                WHERE
                    id = $8
                RETURNING 
                    id,
                    name, 
                    description, 
                    base_url, 
                    redirect_endpoint, 
                    logo_endpoint, 
                    jwt_secret, 
                    jwt_seconds_to_expire, 
                    created_at, 
                    owner_id",
            )
            .bind(self.name.clone())
            .bind(self.description.clone())
            .bind(self.base_url.clone())
            .bind(self.redirect_endpoint.clone())
            .bind(self.logo_endpoint.clone())
            .bind(self.jwt_secret.clone())
            .bind(self.jwt_seconds_to_expire.clone())
            .bind(self.id)
            .fetch_one(&state.db_pool)
            .await
            .map_err(|error| {
                error!("Updating app {:?} -> {:?}", self, error);
                AppError::NotFound
            })?;

            Ok(app)
        }
    }
}
