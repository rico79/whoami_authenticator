mod connection;
mod hello;
mod index;

use axum::{routing::get, Router};
use sqlx::PgPool;
use tower_http::services::ServeDir;

#[derive(Clone)]
struct AppState {
    db_pool: PgPool,
}

// App HTTP server
#[shuttle_runtime::main]
async fn http_server(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://devapp:{secrets.DB_PASSWORD}@localhost:5432/authenticator"
    )]
    db_pool: PgPool,
) -> shuttle_axum::ShuttleAxum {
    // Prepare the app state
    let state = AppState { db_pool };

    // Define router with state for http server
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
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state);

    Ok(router.into())
}
