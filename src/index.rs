use askama_axum::{IntoResponse, Template};

#[derive(Template)]
#[template(path = "index.html")]
pub struct PageTemplate {}

pub async fn get() -> impl IntoResponse {
    PageTemplate {}
}
