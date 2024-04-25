use std::fmt;

use sqlx::types::time::OffsetDateTime;

/// DateTime struct
#[derive(Clone, Debug)]
pub struct DateTime {
    timestamp: i64,
}

/// Generate DateTime from sqlx OffsetDateTime
impl From<OffsetDateTime> for DateTime {
    fn from(offset_date_time: OffsetDateTime) -> Self {
        DateTime {
            timestamp: offset_date_time.unix_timestamp(),
        }
    }
}

/// Format DateTime
impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            chrono::DateTime::from_timestamp(self.timestamp, 0)
                .unwrap_or_default()
                .with_timezone(&chrono::Local).format("%d/%m/%Y à %H:%M")
        )
    }
}
