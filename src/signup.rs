use askama_axum::Template;

#[derive(Template)]
#[template(path = "signup.html")]
pub struct PageTemplate {}

pub async fn get_handler() -> PageTemplate {
    PageTemplate {}
}

pub async fn submit_handler() -> PageTemplate {
    PageTemplate {}
}
