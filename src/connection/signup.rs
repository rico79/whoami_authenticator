use askama_axum::Template;

#[derive(Template)]
#[template(path = "connect/signup.html")]
pub struct PageTemplate {}

pub async fn get() -> PageTemplate {
    PageTemplate {}
}

pub async fn post() -> PageTemplate {
    PageTemplate {}
}
