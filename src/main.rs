mod auth;
mod hello;
mod index;
mod users;
mod utils;

use std::convert::Infallible;
use std::fmt::Debug;

use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    routing::get,
    Router,
};
use utils::email::AppMailer;
use shuttle_runtime::{CustomError, SecretStore};
use sqlx::PgPool;
use tower_http::services::ServeDir;

/// App state
/// Data that can be used in the entire app
#[derive(Clone, Debug)]
pub struct AppState {
    app_url: String,
    db_pool: PgPool,
    mailer: AppMailer,
    jwt_secret: String,
}

/// Implement FromRequestParts
/// FromRequestParts allows us to use the AppState without consuming the request
#[async_trait]
impl<S> FromRequestParts<S> for AppState
where
    Self: FromRef<S>,
    S: Send + Sync + Debug,
{
    type Rejection = Infallible;

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::from_ref(state))
    }
}

/// Main function
/// init and serve the app
#[shuttle_runtime::main]
async fn http_server(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://devapp:{secrets.DB_PASSWORD}@localhost:5432/authenticator"
    )]
    db_pool: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    // Init or update the database (migrations)
    sqlx::migrate!()
        .run(&db_pool)
        .await
        .map_err(CustomError::new)?;

    // Init the mailer
    let mailer = AppMailer::new(&secrets);

    // Set the app state
    let state = AppState {
        app_url: secrets.get("APP_URL").unwrap(),
        db_pool,
        mailer,
        jwt_secret: secrets.get("JWT_SECRET").unwrap(),
    };

    // Define router with state for http server
    let router = Router::new()
        .route("/", get(index::get))
        .route("/hello", get(hello::get))
        .route("/confirm", get(users::confirm::get))
        .route("/signup", get(auth::signup::get).post(auth::signup::post))
        .route("/signin", get(auth::signin::get).post(auth::signin::post))
        .route("/signout", get(auth::signout::get))
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state);

    Ok(router.into())
}
