use askama_axum::{IntoResponse, Template};

#[derive(Template)]
#[template(path = "general/public_page.html")]
pub struct PublicPage {}

pub async fn get_handler() -> impl IntoResponse {
    PublicPage {}
}
