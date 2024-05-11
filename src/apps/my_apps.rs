use askama_axum::{IntoResponse, Template};
use axum::extract::State;

use crate::{apps::App, general::navbar::NavBarBlock, utils::jwt::IdClaims, AppState};

#[derive(Template)]
#[template(path = "apps/my_apps_page.html")]
pub struct MyAppsPage {
    navbar: NavBarBlock,
    my_apps: Vec<App>,
}

pub async fn get_handler(claims: IdClaims, State(state): State<AppState>) -> impl IntoResponse {
    MyAppsPage {
        navbar: NavBarBlock::from(Some(claims.clone())),
        my_apps: App::select_own_apps(&state, &claims).await.unwrap(),
    }
}
