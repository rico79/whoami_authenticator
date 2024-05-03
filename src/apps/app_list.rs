use askama::Template;

use super::App;

#[derive(Template)]
#[template(path = "apps/app_list_block.html")]
pub struct AppListBlock {
    pub apps: Vec<App>,
    pub can_add: bool,
    pub back_url: String,
}
