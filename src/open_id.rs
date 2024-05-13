use askama_axum::IntoResponse;

use crate::auth::IdSession;

pub async fn authorize_handler(id_session: IdSession) -> impl IntoResponse {
    format!(
        "user id: {} and expires in {} seconds",
        id_session.user_id, id_session.seconds_to_expire
    )
}