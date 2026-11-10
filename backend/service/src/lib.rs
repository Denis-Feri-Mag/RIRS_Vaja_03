pub mod auth;
pub mod categories;
pub mod error;
pub mod jobs;
pub mod providers;
pub mod quotes;
pub mod seed;
pub mod workflow;

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

pub(crate) fn dec(v: f64) -> rust_decimal::Decimal {
    use rust_decimal::prelude::FromPrimitive;
    rust_decimal::Decimal::from_f64(v).unwrap_or_default().round_dp(2)
}

pub(crate) fn f(v: rust_decimal::Decimal) -> f64 {
    use rust_decimal::prelude::ToPrimitive;
    v.to_f64().unwrap_or_default()
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
