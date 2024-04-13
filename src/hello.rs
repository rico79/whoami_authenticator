use std::collections::HashMap;

use askama_axum::{IntoResponse, Template};
use axum::extract::Query;

#[derive(Template)]
#[template(path = "hello.html")]
pub struct PageTemplate {
    pub name: String,
}

pub async fn get(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let name = params.get("name").unwrap_or(&"you".to_owned()).to_string();
    PageTemplate { name }
}
