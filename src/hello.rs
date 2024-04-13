use askama_axum::{IntoResponse, Template};
use axum::extract::Query;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "hello.html")]
pub struct PageTemplate {
    pub name: String,
}

#[derive(Deserialize)]
pub struct RequestQuery {
    pub name: String,
}

pub async fn get(query: Option<Query<RequestQuery>>) -> impl IntoResponse {
    match query {
        Some(Query(params)) => PageTemplate { name: params.name },
        None => PageTemplate { name: "you".to_owned() },
    }
}
