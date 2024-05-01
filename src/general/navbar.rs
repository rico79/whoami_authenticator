use askama::Template;

use crate::auth::IdTokenClaims;

/// Message Struct
#[derive(Clone, Debug, Template)]
#[template(path = "general/navbar.html")]
pub struct NavBarTemplate {
    pub claims: Option<IdTokenClaims>,
}
