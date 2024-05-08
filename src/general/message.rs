use askama::Template;

#[derive(Clone, Debug)]
pub enum Level {
    Success,
    Error,
    Info,
}

#[derive(Clone, Debug, Template)]
#[template(path = "general/message_block.html")]
pub struct MessageBlock {
    header: String,
    level: Level,
    body: String,
}

impl MessageBlock {
    pub fn empty() -> Self {
        Self::new(Level::Info, "", "")
    }

    pub fn new(level: Level, header: &str, body: &str) -> Self {
        Self {
            header: header.into(),
            body: body.into(),
            level,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.header.len() <= 0 && self.body.len() <= 0
    }
}
