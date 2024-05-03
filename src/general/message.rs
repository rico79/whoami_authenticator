use std::fmt;

use askama::Template;

#[derive(Clone, Debug)]
pub enum Level {
    Success,
    Error,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            Level::Success => "success",
            Level::Error => "negative",
        };

        write!(f, "{}", message)
    }
}

#[derive(Clone, Debug, Template)]
#[template(path = "general/message_block.html")]
pub struct MessageBlock {
    header: String,
    level: Level,
    body: String,
    is_closeable: bool,
}

impl MessageBlock {
    pub fn empty() -> Self {
        Self {
            header: "".to_owned(),
            level: Level::Success,
            body: "".to_owned(),
            is_closeable: false,
        }
    }

    pub fn closeable(level: Level, header: &str, body: &str) -> Self {
        Self {
            level,
            header: header.to_owned(),
            body: body.to_owned(),
            is_closeable: true,
        }
    }

    pub fn permanent(level: Level, header: &str, body: &str) -> Self {
        Self {
            level,
            header: header.to_owned(),
            body: body.to_owned(),
            is_closeable: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.header.len() <= 0 && self.body.len() <= 0
    }
}
