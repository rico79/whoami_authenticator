use askama_axum::{IntoResponse, Template};
use axum::extract::Query;
use serde::Deserialize;

/** Template
 * HTML page definition with dynamic data
 */
#[derive(Template)]
#[template(path = "hello.html")]
pub struct PageTemplate {
    pub name: String,
}

/** Query parameters definition
 * HTTP parameters used for the get Handler
 */
#[derive(Deserialize)]
pub struct QueryParams {
    name: Option<String>,
}

/** Get handler
 * Returns the page using the dedicated HTML template
 */
pub async fn get(Query(params): Query<QueryParams>) -> impl IntoResponse {
    PageTemplate { name: params.name.unwrap_or("toi".to_owned()) }
}
