use askama_axum::{IntoResponse, Template};

#[derive(Template)]
#[template(path = "connect/signin.html")]
pub struct PageTemplate {}

pub async fn get() -> impl IntoResponse {
    PageTemplate {}
}

pub async fn post() -> impl IntoResponse {
    PageTemplate {}
}
