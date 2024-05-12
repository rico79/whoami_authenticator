use askama::Template;

use crate::auth::IdSession;

use super::navbar::NavBarBlock;

#[derive(Template)]
#[template(path = "whoami_page.html")]
pub struct WhoAmIPage {
    navbar: NavBarBlock,
    id_session: Option<IdSession>,
}

pub async fn get_handler(id_session: Option<IdSession>) -> WhoAmIPage {
    WhoAmIPage {
        navbar: NavBarBlock::from(id_session.clone()),
        id_session,
    }
}
