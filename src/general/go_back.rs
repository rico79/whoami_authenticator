use askama::Template;

#[derive(Template)]
#[template(path = "general/go_back_button.html")]
pub struct GoBackButton {
    pub back_url: String,
}
