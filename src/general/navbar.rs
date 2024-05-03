use askama::Template;

use crate::auth::IdTokenClaims;

#[derive(Clone, Debug, Template)]
#[template(path = "general/navbar_block.html")]
pub struct NavBarBlock {
    pub claims: Option<IdTokenClaims>,
}
