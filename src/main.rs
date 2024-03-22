use askama_axum::Template;
use axum::{routing::get, Router};

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate {
    name: String,
}

async fn hello_world() -> HelloTemplate {
    HelloTemplate { name: "world".to_string() }
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new().route("/", get(hello_world));

    Ok(router.into())
}
