use std::collections::HashMap;

use askama_axum::{IntoResponse, Template};
use axum::extract::Query;

#[derive(Template)]
#[template(path = "hello.html")]
pub struct PageTemplate {
    pub name: String,
}

pub async fn get(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    if let Some(name) = params.get("name") {
        PageTemplate {
            name: name.to_string(),
        }
    } else {
        PageTemplate {
            name: "you".to_owned(),
        }
    }
}
