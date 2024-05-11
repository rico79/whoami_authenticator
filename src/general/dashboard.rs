use askama_axum::{IntoResponse, Template};
use axum::extract::State;

use crate::{apps::App, utils::jwt::IdClaims, AppState};

use super::navbar::NavBarBlock;

#[derive(Template)]
#[template(path = "general/dashboard_page.html")]
pub struct HomePage {
    navbar: NavBarBlock,
    own_apps: Vec<App>,
}

pub async fn get_handler(claims: IdClaims, State(state): State<AppState>) -> impl IntoResponse {
    HomePage {
        navbar: NavBarBlock {
            claims: Some(claims.clone()),
        },
        own_apps: App::select_own_apps(&state, &claims).await.unwrap(),
    }
}
