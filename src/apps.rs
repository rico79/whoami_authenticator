pub mod app;
pub mod my_apps;

use axum::response::Redirect;
use http::Uri;
use shuttle_runtime::SecretStore;
use sqlx::{
    types::{time::OffsetDateTime, Uuid},
    FromRow,
};
use tracing::log::error;

use crate::{auth::IdSession, general::AuthenticatorError, AppState};

#[derive(Clone, Debug, FromRow)]
pub struct App {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub base_url: String,
    redirect_endpoint: String,
    logo_endpoint: String,
    pub jwt_secret: String,
    pub jwt_seconds_to_expire: i32,
    pub created_at: OffsetDateTime,
    pub owner_id: Option<Uuid>,
}

impl App {
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
            created_at: OffsetDateTime::now_utc(),
            owner_id: Some(owner_id.clone()),
        }
    }

    pub fn is_new(&self) -> bool {
        self.id < 0
    }

    pub fn init_authenticator_app(secrets: &SecretStore) -> Self {
        Self {
            id: 0,
            name: secrets.get("APP_NAME").unwrap(),
            description: "Gère la connexion de vos utilisateurs pour vos apps".to_owned(),
            base_url: secrets.get("APP_URL").unwrap(),
            redirect_endpoint: "".to_owned(),
            logo_endpoint: "/assets/images/logo.png".to_owned(),
            jwt_secret: secrets.get("JWT_SECRET").unwrap(),
            jwt_seconds_to_expire: secrets.get("JWT_EXPIRE_SECONDS").unwrap().parse().unwrap(),
            created_at: OffsetDateTime::now_utc(),
            owner_id: None,
        }
    }

    pub fn is_authenticator_app(&self) -> bool {
        self.id == 0
    }

    pub fn is_owned_by(&self, user_id: Uuid) -> bool {
        match self.owner_id.clone() {
            Some(owner_id) => owner_id == user_id.clone(),
            None => false,
        }
    }

    pub fn can_be_updated_by(&self, user_id: Uuid) -> bool {
        !self.is_authenticator_app() && self.is_owned_by(user_id)
    }

    pub async fn select_app_or_authenticator(state: &AppState, app_id: i32) -> Self {
        Self::select_from_app_id(state, app_id)
            .await
            .unwrap_or(state.authenticator_app.clone())
    }

    pub fn domain(&self) -> Result<String, AuthenticatorError> {
        let uri = self
            .base_url
            .parse::<Uri>()
            .map_err(|_| AuthenticatorError::AppInvalidUri)?;

        let authority = uri.authority().ok_or(AuthenticatorError::AppInvalidUri)?;

        Ok(authority.host().to_string())
    }

    pub fn redirect_url(&self) -> String {
        self.url_to_endpoint(&self.redirect_endpoint)
    }

    fn url_to_endpoint(&self, endpoint: &String) -> String {
        match (self.base_url.ends_with("/"), endpoint.starts_with("/")) {
            (true, true) => format!("{}{}", self.base_url, &endpoint[1..]),
            (true, false) => format!("{}{}", self.base_url, endpoint),
            (false, true) => format!("{}{}", self.base_url, endpoint),
            (false, false) => format!("{}/{}", self.base_url, endpoint),
        }
    }

    pub fn redirect_to(&self) -> Redirect {
        self.redirect_to_endpoint(None)
    }

    pub fn redirect_to_endpoint(&self, endpoint: Option<String>) -> Redirect {
        match endpoint {
            Some(endpoint) => Redirect::to(&self.url_to_endpoint(&endpoint)),
            None => Redirect::to(&self.redirect_url()),
        }
    }

    pub fn logo_url(&self) -> String {
        if self.base_url.len() > 0 && self.logo_endpoint.len() > 0 {
            format!("{}{}", &self.base_url, &self.logo_endpoint)
        } else {
            "/assets/images/app.png".to_owned()
        }
    }

    pub async fn select_own_apps(
        state: &AppState,
        id_session: &IdSession,
    ) -> Result<Vec<Self>, AuthenticatorError> {
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
                owner_id = $1
            ORDER BY
                name",
        )
        .bind(id_session.user_id)
        .fetch_all(&state.db_pool)
        .await
        .map_err(|error| {
            error!(
                "Selecting apps for owner {} -> {:?}",
                id_session.user_id, error
            );
            AuthenticatorError::DatabaseError
        })?;

        let user_is_authenticator_app_owner = id_session.mail == state.owner_mail;

        if user_is_authenticator_app_owner {
            apps.push(state.authenticator_app.clone())
        }

        Ok(apps)
    }

    pub async fn select_from_app_id(
        state: &AppState,
        app_id: i32,
    ) -> Result<Self, AuthenticatorError> {
        let is_authenticator_app = app_id == state.authenticator_app.id;

        if is_authenticator_app {
            return Ok(state.authenticator_app.clone());
        }

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
            AuthenticatorError::AppNotFound
        })?;

        Ok(app)
    }

    pub async fn save(
        &self,
        state: &AppState,
        id_session: &IdSession,
    ) -> Result<Self, AuthenticatorError> {
        if self.is_authenticator_app() {
            return Ok(state.authenticator_app.clone());
        }

        if self.is_new() {
            let inserted_app: App = sqlx::query_as(
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
            .bind(id_session.user_id)
            .fetch_one(&state.db_pool)
            .await
            .map_err(|error| {
                error!("Inserting app {:?} -> {:?}", self, error);
                AuthenticatorError::AppNotFound
            })?;

            Ok(inserted_app)
        } else {
            let updated_app: App = sqlx::query_as(
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
                AuthenticatorError::AppNotFound
            })?;

            Ok(updated_app)
        }
    }
}
