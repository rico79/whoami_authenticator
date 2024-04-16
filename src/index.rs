use askama_axum::{IntoResponse, Template};

/** Template
 * HTML page definition with dynamic data
 */
#[derive(Template)]
#[template(path = "index.html")]
pub struct PageTemplate {}

/** Get handler
 * Returns the page using the dedicated HTML template
 */
pub async fn get() -> impl IntoResponse {
    PageTemplate {}
}
