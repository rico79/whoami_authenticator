use askama_axum::IntoResponse;
use axum_extra::extract::CookieJar;

use super::IdSession;

pub async fn get_handler(cookies: CookieJar) -> impl IntoResponse {
    IdSession::remove_and_redirect_to(cookies, "/")
}
