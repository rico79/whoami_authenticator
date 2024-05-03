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
use utils::mail::AppMailer;

/// App state
/// Data that can be used in the entire app
#[derive(Clone, Debug)]
pub struct AppState {
    owner_mail: String,
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
    sqlx::migrate!()
        .run(&db_pool)
        .await
        .map_err(CustomError::new)?;

    let state = AppState {
        owner_mail: secrets.get("OWNER_MAIL").unwrap(),
        authenticator_app: App::init_authenticator_app(&secrets),
        db_pool,
        mailer: AppMailer::new(&secrets),
    };

    let router = Router::new()
        .route("/", get(general::public::get))
        .route("/home", get(general::home::get))
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
