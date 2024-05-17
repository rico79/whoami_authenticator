use askama::Template;
use axum::extract::State;

use crate::{auth::IdSession, AppState};

use super::navbar::NavBarBlock;

#[derive(Template)]
#[template(path = "whoami_page.html")]
pub struct WhoAmIPage {
    navbar: NavBarBlock,
    id_session: Option<IdSession>,
}

pub async fn get_handler(
    id_session: Option<IdSession>,
    State(state): State<AppState>,
) -> WhoAmIPage {
    WhoAmIPage {
        navbar: NavBarBlock::from(&state, id_session.clone()),
        id_session,
    }
}
