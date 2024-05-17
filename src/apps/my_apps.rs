use askama_axum::{IntoResponse, Template};
use axum::extract::State;

use crate::{apps::App, auth::IdSession, general::navbar::NavBarBlock, AppState};

#[derive(Template)]
#[template(path = "apps/my_apps_page.html")]
pub struct MyAppsPage {
    navbar: NavBarBlock,
    my_apps: Vec<App>,
}

pub async fn get_handler(
    id_session: IdSession,
    State(state): State<AppState>,
) -> impl IntoResponse {
    MyAppsPage {
        navbar: NavBarBlock::from(&state, Some(id_session.clone())),
        my_apps: App::select_own_apps(&state, &id_session).await.unwrap(),
    }
}
