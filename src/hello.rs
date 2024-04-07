use askama_axum::Template;
use axum::extract::Query;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "hello.html")]
pub struct PageTemplate {
    name: String,
}

#[derive(Deserialize)]
pub struct RequestQuery {
    pub name: String,
}

pub async fn get(Query(params): Query<RequestQuery>) -> PageTemplate {
    PageTemplate { name: params.name }
}
