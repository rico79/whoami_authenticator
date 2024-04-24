use axum::response::Redirect;

/// App redirection
/// Redirect to the app welcome page when signed-in
/// Get the app id to redirect to
pub fn redirect_to_app_welcome(app_id: Option<String>) -> Redirect {
    // If there is no app id redirect to authenticator welcome page
    if app_id.is_some_and(|app_id| app_id.len() > 0) {
        // Get app welcome page

        // Redirect
        Redirect::to("/welcome")
    } else {
        // Authenticator welcome page
        Redirect::to("/welcome")
    }
}