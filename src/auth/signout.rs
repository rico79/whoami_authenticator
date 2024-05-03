use askama_axum::IntoResponse;
use axum_extra::extract::CookieJar;

use super::remove_id_token_and_redirect;

pub async fn get_handler(cookies: CookieJar) -> impl IntoResponse {
    remove_id_token_and_redirect(cookies, "/")
}
