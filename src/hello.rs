use askama_axum::Template;
use axum::extract::Query;
use serde::Deserialize;

#[derive(Template)]
#[template(path = "hello.html")]
pub struct HelloTemplate {
    name: String,
}

#[derive(Deserialize)]
pub struct HelloQuery {
    pub name: String,
}

pub async fn handler(Query(params): Query<HelloQuery>) -> HelloTemplate {
    HelloTemplate { name: params.name }
}
