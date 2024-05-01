use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::{Query, State},
    Form,
};
use serde::Deserialize;

use crate::{
    auth::IdTokenClaims, general::navbar::NavBarTemplate, utils::date_time::DateTime, AppState,
};

use super::App;

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "apps/app.html")]
pub struct PageTemplate {
    navbar: NavBarTemplate,
    app: Option<App>,
    read_only: bool,
}

impl PageTemplate {
    /// Init page from App
    fn from(claims: &IdTokenClaims, app: Option<App>) -> Result<Self, Self> {
        Ok(PageTemplate {
            navbar: NavBarTemplate {
                claims: Some(claims.clone()),
            },
            app: app.clone(),
            read_only: !app.unwrap_or(App::new()).can_be_updated_by(claims.user_id()),
        })
    }

    /// Init page from app_id
    async fn from_id(
        state: &AppState,
        claims: &IdTokenClaims,
        app_id: Option<i64>,
    ) -> Result<Self, Self> {
        match app_id {
            // Get app from id
            Some(app_id) => Self::from(claims, App::select_from_app_id(&state, app_id).await.ok()),
            // No id means new app to create
            None => Err(PageTemplate {
                navbar: NavBarTemplate {
                    claims: Some(claims.clone()),
                },
                app: Some(App::new()),
                read_only: false,
            }),
        }
    }
}

/// Query parameters definition
/// HTTP parameters used for the get Handler
#[derive(Deserialize)]
pub struct QueryParams {
    id: Option<i64>,
}

/// Get handler
/// Returns the page using the dedicated HTML template
pub async fn get(
    claims: IdTokenClaims,
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    PageTemplate::from_id(&state, &claims, params.id).await
}

/// Post form
/// Data expected from the form
#[derive(Deserialize)]
pub struct PostForm {
    id: i64,
    name: Option<String>,
    description: Option<String>,
    base_url: Option<String>,
    redirect_endpoint: Option<String>,
    logo_endpoint: Option<String>,
    jwt_secret: Option<String>,
    jwt_seconds_to_expire: Option<i64>,
}

/// Post handler
/// Returns the page using the dedicated HTML template
pub async fn post(
    claims: IdTokenClaims,
    State(state): State<AppState>,
    Form(form): Form<PostForm>,
) -> impl IntoResponse {
    // Check if read only (= name is missing)
    match form.name {
        Some(name) => PageTemplate::from(
            &claims,
            App {
                id: form.id,
                name,
                description: form.description.unwrap_or("".to_owned()),
                base_url: form.base_url.unwrap_or("".to_owned()),
                redirect_endpoint: form.redirect_endpoint.unwrap_or("".to_owned()),
                logo_endpoint: form.logo_endpoint.unwrap_or("".to_owned()),
                jwt_secret: form.jwt_secret.unwrap_or("".to_owned()),
                jwt_seconds_to_expire: form.jwt_seconds_to_expire.unwrap_or(0),
                created_at: DateTime::default(),
                owner_id: Some(claims.user_id()),
            }
            .save(&state, &claims)
            .await
            .ok(),
        ),
        None => PageTemplate::from_id(&state, &claims, Some(form.id)).await,
    }
}
