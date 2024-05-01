mod apps;
mod auth;
mod general;
mod users;
mod utils;

use std::convert::Infallible;
use std::fmt::Debug;

use apps::App;
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
    routing::{get, post},
    Router,
};
use shuttle_runtime::{CustomError, SecretStore};
use sqlx::PgPool;
use tower_http::services::{ServeDir, ServeFile};
use utils::{date_time::DateTime, email::AppMailer};

/// App state
/// Data that can be used in the entire app
#[derive(Clone, Debug)]
pub struct AppState {
    authenticator_app: App,
    db_pool: PgPool,
    mailer: AppMailer,
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

    // Init authenticator app
    let authenticator_app = App::init_authenticator_app(
        &db_pool,
        secrets.get("APP_URL").unwrap(),
        secrets.get("JWT_SECRET").unwrap(),
        secrets.get("JWT_EXPIRE_SECONDS").unwrap().parse().unwrap(),
        DateTime::from_timestamp(1712899091),
        secrets.get("MAIL_USER_NAME").unwrap(),
    ).await;

    // Set the app state
    let state = AppState {
        authenticator_app,
        db_pool,
        mailer: AppMailer::new(&secrets),
    };

    // Define router with state for http server
    let router = Router::new()
        .route("/", get(general::home::get))
        .route("/welcome", get(general::welcome::get))
        .route("/confirm", get(users::confirm::get))
        .route("/signup", get(auth::signup::get).post(auth::signup::post))
        .route("/signin", get(auth::signin::get).post(auth::signin::post))
        .route("/signout", get(auth::signout::get))
        .route(
            "/profile",
            get(users::profile::get).post(users::profile::update_profile),
        )
        .route("/password", post(users::profile::update_password))
        .route("/app", get(apps::app::get).post(apps::app::post))
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/favicon.ico", ServeFile::new("assets/favicon.ico"))
        .with_state(state);

    Ok(router.into())
}
