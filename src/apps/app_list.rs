use askama::Template;

use super::App;

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "apps/app_list.html")]
pub struct AppListTemplate {
    pub apps: Vec<App>,
    pub can_add: bool,
}
