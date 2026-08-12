use chrono::{DateTime, NaiveTime, Timelike, Utc};

use crate::time::{BEIJING_TIMEZONE, beijing_local_to_utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScanWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

pub fn next_scheduled_run(now_utc: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let local_now = now_utc.with_timezone(&BEIJING_TIMEZONE);
    let local_date = local_now.date_naive();
    let local_time = local_now.time();
    let monitor_start = NaiveTime::from_hms_opt(8, 0, 0)?;
    let monitor_end = NaiveTime::from_hms_opt(23, 0, 0)?;

    if local_time < monitor_start {
        return beijing_local_to_utc(local_date, monitor_start);
    }

    if local_time < monitor_end {
        let next_hour = NaiveTime::from_hms_opt(local_time.hour() + 1, 0, 0)?;
        return beijing_local_to_utc(local_date, next_hour);
    }

    beijing_local_to_utc(local_date.succ_opt()?, monitor_start)
}

pub fn catchup_window(run_at_utc: DateTime<Utc>) -> Option<ScanWindow> {
    let local_run = run_at_utc.with_timezone(&BEIJING_TIMEZONE);
    let catchup_end = NaiveTime::from_hms_opt(8, 0, 0)?;
    if local_run.time() != catchup_end {
        return None;
    }

    let previous_date = local_run.date_naive().pred_opt()?;
    let start = beijing_local_to_utc(previous_date, NaiveTime::MIN)?;
    let end = beijing_local_to_utc(local_run.date_naive(), catchup_end)?;
    Some(ScanWindow { start, end })
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};

    use super::{catchup_window, next_scheduled_run};

    fn at(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn scheduler_returns_0800_after_075959_beijing_time() {
        assert_eq!(
            next_scheduled_run(at("2026-08-11T23:59:59Z")),
            Some(at("2026-08-12T00:00:00Z"))
        );
    }

    #[test]
    fn scheduler_returns_next_hour_after_exact_0800_run() {
        assert_eq!(
            next_scheduled_run(at("2026-08-12T00:00:00Z")),
            Some(at("2026-08-12T01:00:00Z"))
        );
    }

    #[test]
    fn scheduler_returns_next_day_0800_after_2300() {
        assert_eq!(
            next_scheduled_run(at("2026-08-12T15:00:00Z")),
            Some(at("2026-08-13T00:00:00Z"))
        );
    }

    #[test]
    fn catchup_window_crosses_month_and_year_boundaries() {
        let month = catchup_window(at("2026-03-01T00:00:00Z")).unwrap();
        assert_eq!(month.start, at("2026-02-27T16:00:00Z"));
        assert_eq!(month.end, at("2026-03-01T00:00:00Z"));

        let year = catchup_window(at("2027-01-01T00:00:00Z")).unwrap();
        assert_eq!(year.start, at("2026-12-30T16:00:00Z"));
        assert_eq!(year.end, at("2027-01-01T00:00:00Z"));
    }
}
