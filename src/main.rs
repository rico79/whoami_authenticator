mod hello;
mod index;
mod connection;

use axum::{routing::get, Router};
use tower_http::services::ServeDir;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(index::get_handler))
        .route("/hello", get(hello::get_handler))
        .route(
            "/signup",
            get(connection::signup::get_handler).post(connection::signup::submit_handler),
        )
        .route(
            "/signin",
            get(connection::signin::get_handler).post(connection::signin::submit_handler),
        )
        .nest_service("/assets", ServeDir::new("assets"));

    Ok(router.into())
}
