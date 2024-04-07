use askama_axum::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct PageTemplate {}

pub async fn get() -> PageTemplate {
    PageTemplate {}
}
