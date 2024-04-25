use askama_axum::{IntoResponse, Template};

use crate::auth::IdTokenClaims;

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "general/welcome.html")]
pub struct PageTemplate {
    claims: Option<IdTokenClaims>,
    name: String,
}

/// Get handler
/// Returns the page using the dedicated HTML template
pub async fn get(claims: IdTokenClaims) -> impl IntoResponse {
    PageTemplate {
        claims: Some(claims.clone()),
        name: claims.name.clone(),
    }
}
