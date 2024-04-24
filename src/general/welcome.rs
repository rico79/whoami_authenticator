use askama_axum::{IntoResponse, Template};

use crate::auth::JWTClaims;

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "general/welcome.html")]
pub struct PageTemplate {
    claims: JWTClaims,
    name: String,
}

/// Get handler
/// Returns the page using the dedicated HTML template
pub async fn get(claims: JWTClaims) -> impl IntoResponse {
    PageTemplate {
        name: claims.name.clone(),
        claims: claims,
    }
}
