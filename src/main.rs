mod hello;

use axum::{routing::get, Router};

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new().route("/hello", get(hello::hello_world));

    Ok(router.into())
}
