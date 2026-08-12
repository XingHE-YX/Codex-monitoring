use chrono::{DateTime, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::{Asia::Shanghai, Tz};

pub const BEIJING_TIMEZONE: Tz = Shanghai;

pub fn beijing_local_to_utc(date: NaiveDate, time: NaiveTime) -> Option<DateTime<Utc>> {
    BEIJING_TIMEZONE
        .from_local_datetime(&date.and_time(time))
        .single()
        .map(|value| value.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveTime};

    use super::beijing_local_to_utc;

    #[test]
    fn beijing_local_time_is_converted_to_utc() {
        let date = NaiveDate::from_ymd_opt(2026, 8, 12).unwrap();
        let time = NaiveTime::from_hms_opt(8, 0, 0).unwrap();

        assert_eq!(
            beijing_local_to_utc(date, time).unwrap().to_rfc3339(),
            "2026-08-12T00:00:00+00:00"
        );
    }
}
