use askama_axum::IntoResponse;
use axum::response::Redirect;

use super::IdSession;

pub async fn get_handler(id_session: Option<IdSession>) -> impl IntoResponse {
    let redirect_to = "/";

    if let Some(id_session) = id_session {
        id_session
            .remove_and_redirect_to(redirect_to)
            .into_response()
    } else {
        Redirect::to(redirect_to).into_response()
    }
}
