pub mod home;
pub mod welcome;

/// Message Struct
#[derive(Clone, Debug)]
pub struct PageMessage {
    pub header: String,
    pub level: String,
    pub body: String,
}

impl PageMessage {
    /// Empty message
    pub fn empty() -> Self {
        Self::from("".to_owned(), "".to_owned(), "".to_owned())
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.header.len() <= 0 && self.level.len() <= 0 && self.body.len() <= 0
    }

    /// From
    pub fn from(header: String, level: String, body: String) -> Self {
        PageMessage {
            header: header,
            level: level,
            body: body,
        }
    }

    /// From
    pub fn from_body(level: String, body: String) -> Self {
        Self::from("".to_owned(), level, body)
    }
}
