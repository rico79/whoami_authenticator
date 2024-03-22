use askama_axum::Template;
use axum::{extract::Query, routing::get, Router};
use serde::Deserialize;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new().route("/hello", get(hello_world));

    Ok(router.into())
}

// Hello world handler
#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate {
    name: String,
}

#[derive(Deserialize)]
pub struct HelloQuery {
	pub name: String,
}

async fn hello_world(Query(params): Query<HelloQuery>) -> HelloTemplate {
    HelloTemplate {
        name: params.name,
    }
}