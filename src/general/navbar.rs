use askama::Template;

use crate::{apps::App, auth::IdSession, AppState};

#[derive(Clone, Debug, Template)]
#[template(path = "general/navbar_block.html")]
pub struct NavBarBlock {
    id_session: Option<IdSession>,
    can_create_app: bool,
}

impl NavBarBlock {
    pub fn from(state: &AppState, id_session: Option<IdSession>) -> Self {
        let can_create_app = match id_session.clone() {
            Some(id_session) => App::can_be_created_by(state, id_session.mail),
            None => false,
        };

        Self {
            id_session,
            can_create_app,
        }
    }
}
