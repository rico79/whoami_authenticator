use askama_axum::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct PageTemplate {
    name: String,
}

pub async fn handler() -> PageTemplate {
    PageTemplate {
        name: "Rico".to_owned(),
    }
}
