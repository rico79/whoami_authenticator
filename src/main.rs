mod hello;
mod index;

use axum::{routing::get, Router};

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(index::handler))
        .route("/hello", get(hello::handler));

    Ok(router.into())
}
