use askama_axum::{IntoResponse, Template};
use axum::extract::State;

use crate::{
    apps::{app_list::AppListBlock, App},
    auth::IdTokenClaims,
    AppState,
};

use super::navbar::NavBarBlock;

#[derive(Template)]
#[template(path = "general/home_page.html")]
pub struct HomePage {
    navbar: NavBarBlock,
    own_apps: AppListBlock,
}

pub async fn get_handler(
    claims: IdTokenClaims,
    State(state): State<AppState>,
) -> impl IntoResponse {
    HomePage {
        navbar: NavBarBlock {
            claims: Some(claims.clone()),
        },
        own_apps: AppListBlock {
            apps: App::select_own_apps(&state, &claims).await.unwrap(),
            can_add: true,
            back_url: "/home".to_owned(),
        },
    }
}
