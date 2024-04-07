use askama_axum::Template;

#[derive(Template)]
#[template(path = "connect/signin.html")]
pub struct PageTemplate {}

pub async fn get() -> PageTemplate {
    PageTemplate {}
}

pub async fn post() -> PageTemplate {
    PageTemplate {}
}
