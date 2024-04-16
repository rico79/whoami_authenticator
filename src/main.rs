mod connection;
mod email;
mod hello;
mod index;
mod users;

use axum::{routing::get, Router};
use email::AppMailer;
use shuttle_runtime::{CustomError, SecretStore};
use sqlx::PgPool;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    app_url: String,
    db_pool: PgPool,
    mailer: AppMailer,
}

#[shuttle_runtime::main]
async fn http_server(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://devapp:{secrets.DB_PASSWORD}@localhost:5432/authenticator"
    )]
    db_pool: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    // S
    // Init or update the database (migrations)
    sqlx::migrate!()
        .run(&db_pool)
        .await
        .map_err(CustomError::new)?;

    // Init the mailer
    let mailer = AppMailer::new(&secrets);

    // Set the app state
    let state = AppState { app_url: secrets.get("APP_URL").unwrap(), db_pool, mailer };

    // Define router with state for http server
    let router = Router::new()
        .route("/", get(index::get))
        .route("/hello", get(hello::get))
        .route("/confirm", get(users::confirmation::get))
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
