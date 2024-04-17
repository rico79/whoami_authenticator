use askama_axum::{IntoResponse, Template};

use crate::auth::JWTClaims;

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "hello.html")]
pub struct PageTemplate {
    pub message: String,
}

/// Get handler
/// Returns the page using the dedicated HTML template
pub async fn get(jwt_claims: JWTClaims) -> impl IntoResponse {
    PageTemplate {
        message: jwt_claims.to_string(),
    }
}
