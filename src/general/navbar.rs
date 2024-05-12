use askama::Template;

use crate::auth::IdSession;

#[derive(Clone, Debug, Template)]
#[template(path = "general/navbar_block.html")]
pub struct NavBarBlock {
    id_session: Option<IdSession>,
}

impl NavBarBlock {
    pub fn from(id_session: Option<IdSession>) -> Self {
        Self { id_session }
    }
}
