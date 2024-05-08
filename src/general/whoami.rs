use askama::Template;

use crate::utils::jwt::IdClaims;

#[derive(Template)]
#[template(path = "whoami_page.html")]
pub struct WhoAmIPage {
    claims: Option<IdClaims>,
}

pub async fn get_handler(claims: Option<IdClaims>) -> WhoAmIPage {
    WhoAmIPage { claims }
}
