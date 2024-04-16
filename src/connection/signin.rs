use askama_axum::{IntoResponse, Template};

/** Template
 * HTML page definition with dynamic data
 */
#[derive(Template)]
#[template(path = "connection/signin.html")]
pub struct PageTemplate {}

/** Get handler
 * Returns the page using the dedicated HTML template
 */
pub async fn get() -> impl IntoResponse {
    PageTemplate {}
}

/** Post handler
 * Process the signin form to create a user session and redirect to the expected app
 */
pub async fn post() -> impl IntoResponse {
    PageTemplate {}
}
