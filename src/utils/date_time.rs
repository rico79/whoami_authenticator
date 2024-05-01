use std::fmt;

use sqlx::types::time::OffsetDateTime;

/// DateTime struct
#[derive(Clone, Debug)]
pub struct DateTime {
    timestamp: i64,
}

impl DateTime {
    /// Get Datetime from timestamp
    pub fn from_timestamp(timestamp: i64) -> Self {
        Self { timestamp }
    }

    /// Format following pattern
    pub fn format_to_string(&self, pattern: &str) -> String {
        chrono::DateTime::from_timestamp(self.timestamp, 0)
            .unwrap_or_default()
            .with_timezone(&chrono::Local)
            .format(pattern)
            .to_string()
    }
}

/// Generate DateTime from sqlx OffsetDateTime
impl From<OffsetDateTime> for DateTime {
    fn from(offset_date_time: OffsetDateTime) -> Self {
        DateTime {
            timestamp: offset_date_time.unix_timestamp(),
        }
    }
}

/// DateTime default
impl Default for DateTime {
    fn default() -> Self {
        Self { timestamp: 0 }
    }
}

/// Format DateTime
impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.format_to_string("%d/%m/%Y à %H:%M")
        )
    }
}
