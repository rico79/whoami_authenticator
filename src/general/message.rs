use askama::Template;

/// Message Struct
#[derive(Clone, Debug)]
#[derive(Template)]
#[template(path = "general/message.html")]
pub struct MessageTemplate {
    pub header: String,
    pub level: String,
    pub body: String,
    pub closeable: bool,
}

impl MessageTemplate {
    /// Empty message
    pub fn empty() -> Self {
        Self::from("".to_owned(), "".to_owned(), "".to_owned(), false)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.header.len() <= 0 && self.level.len() <= 0 && self.body.len() <= 0
    }

    /// From
    pub fn from(header: String, level: String, body: String, closeable: bool) -> Self {
        Self {
            header,
            level,
            body,
            closeable,
        }
    }

    /// From
    pub fn from_body(level: String, body: String, closeable: bool) -> Self {
        Self::from("".to_owned(), level, body, closeable)
    }
}
