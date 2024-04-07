mod connection;
mod hello;
mod index;

use axum::{routing::get, Router};
use tower_http::services::ServeDir;

#[shuttle_runtime::main]
async fn http_server() -> shuttle_axum::ShuttleAxum {
    // Define router for http server
    let router = Router::new()
        .route("/", get(index::get))
        .route("/hello", get(hello::get))
        .route(
            "/signup",
            get(connection::signup::get).post(connection::signup::post),
        )
        .route(
            "/signin",
            get(connection::signin::get).post(connection::signin::post),
        )
        .nest_service("/assets", ServeDir::new("assets"));

    Ok(router.into())
}
