use askama::Template;
use askama_axum::IntoResponse;
use axum::{
    extract::{Query, State},
    Form,
};
use serde::Deserialize;
use time::OffsetDateTime;

use crate::{general::navbar::NavBarBlock, utils::jwt::IdClaims, AppState};

use super::App;

#[derive(Template)]
#[template(path = "apps/app_page.html")]
pub struct AppPage {
    navbar: NavBarBlock,
    app: Option<App>,
    read_only: bool,
}

impl AppPage {
    pub fn print_read_only(&self) -> String {
        if self.read_only {
            "readonly".to_owned()
        } else {
            "".to_owned()
        }
    }

    fn from_app(claims: &IdClaims, app: Option<App>) -> Result<Self, Self> {
        match app {
            Some(app) => Ok(AppPage {
                navbar: NavBarBlock::from(Some(claims.clone())),
                app: Some(app.clone()),
                read_only: !app.can_be_updated_by(claims.user_id()),
            }),

            None => Ok(AppPage {
                navbar: NavBarBlock::from(Some(claims.clone())),
                app: app.clone(),
                read_only: !App::new(&claims.user_id()).can_be_updated_by(claims.user_id()),
            }),
        }
    }

    async fn from_app_id(
        state: &AppState,
        claims: &IdClaims,
        app_id: Option<i32>,
    ) -> Result<Self, Self> {
        match app_id {
            Some(app_id) => {
                Self::from_app(claims, App::select_from_app_id(&state, app_id).await.ok())
            }

            None => Err(AppPage {
                navbar: NavBarBlock::from(Some(claims.clone())),
                app: Some(App::new(&claims.user_id())),
                read_only: false,
            }),
        }
    }
}

#[derive(Deserialize)]
pub struct QueryParams {
    id: Option<i32>,
}

pub async fn get_handler(
    claims: IdClaims,
    State(state): State<AppState>,
    Query(params): Query<QueryParams>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    AppPage::from_app_id(&state, &claims, params.id).await
}

#[derive(Deserialize)]
pub struct PostForm {
    id: i32,
    name: Option<String>,
    description: Option<String>,
    base_url: Option<String>,
    redirect_endpoint: Option<String>,
    logo_endpoint: Option<String>,
    jwt_secret: Option<String>,
    jwt_seconds_to_expire: Option<i32>,
}

pub async fn post_handler(
    claims: IdClaims,
    State(state): State<AppState>,
    Form(form): Form<PostForm>,
) -> impl IntoResponse {
    // Check if read only (= name is missing)
    match form.name {
        Some(name) => AppPage::from_app(
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
                created_at: OffsetDateTime::now_utc(),
                owner_id: Some(claims.user_id()),
            }
            .save(&state, &claims)
            .await
            .ok(),
        ),
        None => AppPage::from_app_id(&state, &claims, Some(form.id)).await,
    }
}
