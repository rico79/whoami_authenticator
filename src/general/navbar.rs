use askama::Template;

use crate::utils::jwt::IdClaims;

#[derive(Clone, Debug, Template)]
#[template(path = "general/navbar_block.html")]
pub struct NavBarBlock {
    claims: Option<IdClaims>,
}

impl NavBarBlock {
    pub fn from(claims: Option<IdClaims>) -> Self {
        Self { claims }
    }
}
