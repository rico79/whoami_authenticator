use askama_axum::IntoResponse;
use axum_extra::extract::CookieJar;

use super::remove_session_and_redirect;

/// Get handler
/// Remove session from cookies and redirect to specified url
pub async fn get(cookies: CookieJar) -> impl IntoResponse {
    remove_session_and_redirect(cookies, "/")
}