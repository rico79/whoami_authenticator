use time::format_description::{self, BorrowedFormatItem};

use crate::general::AuthenticatorError;

fn html_date_formatter() -> Vec<BorrowedFormatItem<'static>> {
    format_description::parse("[year]-[month]-[day]").unwrap()
}

pub struct HtmlDate {
    date: String,
}

impl From<String> for HtmlDate {
    fn from(date: String) -> Self {
        Self { date }
    }
}

impl From<&String> for HtmlDate {
    fn from(date: &String) -> Self {
        Self {
            date: date.to_string(),
        }
    }
}

impl TryInto<sqlx::types::time::Date> for HtmlDate {
    type Error = AuthenticatorError;

    fn try_into(self) -> Result<sqlx::types::time::Date, Self::Error> {
        sqlx::types::time::Date::parse(&self.date, &html_date_formatter())
            .map_err(|_| AuthenticatorError::InvalidDate)
    }
}
