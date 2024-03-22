use askama_axum::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    name: String,
}

pub async fn handler() -> IndexTemplate {
    IndexTemplate {
        name: "Rico".to_owned(),
    }
}
