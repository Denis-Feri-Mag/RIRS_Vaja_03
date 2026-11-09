pub mod auth;
pub mod error;

pub use error::AppError;
pub use sea_orm::DatabaseConnection;

/// App time zone, dates without time like earliest_start are meant in Ljubljana.
pub const TIME_ZONE: chrono_tz::Tz = chrono_tz::Europe::Ljubljana;

pub fn today() -> chrono::NaiveDate {
    today_at(chrono::Utc::now())
}

pub fn today_at(now: chrono::DateTime<chrono::Utc>) -> chrono::NaiveDate {
    now.with_timezone(&TIME_ZONE).date_naive()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    #[test]
    fn today_is_ljubljana_date() {
        // 23:30 UTC on 9 Nov is already 10 Nov in Ljubljana
        let t = chrono::Utc.with_ymd_and_hms(2026, 11, 9, 23, 30, 0).unwrap();
        assert_eq!(super::today_at(t).to_string(), "2026-11-10");
        let t = chrono::Utc.with_ymd_and_hms(2026, 11, 9, 22, 30, 0).unwrap();
        assert_eq!(super::today_at(t).to_string(), "2026-11-09");
    }
}
