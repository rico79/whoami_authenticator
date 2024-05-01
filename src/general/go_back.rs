use askama::Template;

/// Template
/// HTML page definition with dynamic data
#[derive(Template)]
#[template(path = "general/go_back.html")]
pub struct GoBackTemplate {
    pub back_url: String,
}