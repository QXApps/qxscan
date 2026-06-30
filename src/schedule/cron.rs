//! ############################################################################
//! @file       cron.rs
//! @company    QuantX, LLC.
//! @author     Phaneendra Bhattiprolu <phanibh@qxapps.net>
//! @date       2026-06-26
//! @brief      Cron expression helpers — named interval expansion and next-run computation.
//!
//! @details
//!
//! ### REVISION HISTORY
//! | Date       | Version | Author                  | Description |
//! |------------|---------|-------------------------|-------------|
//! | 2026-06-02 | 1.0.0   | Phaneendra Bhattiprolu  | Initial implementation. |
//! |            |         |                         |             |
//!
//! ### COMMENTS / NOTES
//! ############################################################################
//! Cron expression helpers and named interval expansion.

use chrono::{DateTime, Datelike, Duration, NaiveTime, Timelike, Utc, Weekday};

use crate::cli::schedule::Interval;

pub fn interval_to_cron(interval: &Interval) -> &'static str {
    match interval {
        Interval::Hourly => "0 * * * *",
        Interval::Daily => "0 2 * * *",
        Interval::Weekly => "0 2 * * 1",
        Interval::Monthly => "0 2 1 * *",
    }
}

pub fn next_run_from_cron(cron_expr: &str, after: &DateTime<Utc>) -> anyhow::Result<DateTime<Utc>> {
    let parts: Vec<&str> = cron_expr.split_whitespace().collect();
    if parts.len() != 5 {
        anyhow::bail!(
            "invalid cron expression: expected 5 fields, got {}",
            parts.len()
        );
    }

    let minute = parse_cron_field(parts[0], 0, 59)?;
    let hour = parse_cron_field(parts[1], 0, 23)?;
    let day_of_month = parse_cron_field(parts[2], 1, 31)?;
    let month = parse_cron_field(parts[3], 1, 12)?;
    let day_of_week = parse_cron_field(parts[4], 0, 7)?;

    let mut candidate = *after + Duration::minutes(1);
    candidate = candidate.with_second(0).unwrap_or(candidate);

    for _ in 0..(365 * 48) {
        if !month_matches(candidate.month() as i32, &month) {
            candidate = advance_to_next_month(candidate);
            continue;
        }
        if !dom_matches(candidate.day() as i32, &day_of_month) {
            candidate += Duration::days(1);
            candidate = candidate.with_hour(0).unwrap_or(candidate);
            candidate = candidate.with_minute(0).unwrap_or(candidate);
            continue;
        }
        if !dow_matches(candidate.weekday(), &day_of_week) {
            candidate += Duration::days(1);
            candidate = candidate.with_hour(0).unwrap_or(candidate);
            candidate = candidate.with_minute(0).unwrap_or(candidate);
            continue;
        }
        if !hour_matches(candidate.hour() as i32, &hour) {
            candidate += Duration::hours(1);
            candidate = candidate.with_minute(0).unwrap_or(candidate);
            continue;
        }
        if !minute_matches(candidate.minute() as i32, &minute) {
            candidate += Duration::minutes(1);
            continue;
        }
        return Ok(candidate);
    }

    anyhow::bail!("could not compute next run time within a reasonable window")
}

fn parse_cron_field(field: &str, min: i32, max: i32) -> anyhow::Result<Vec<i32>> {
    if field == "*" {
        return Ok((-1..=0).collect());
    }
    let mut values = Vec::new();
    for part in field.split(',') {
        if let Some((start, end)) = part.split_once('-') {
            let s: i32 = start.parse()?;
            let e: i32 = end.parse()?;
            for v in s..=e {
                if v >= min && v <= max {
                    values.push(v);
                }
            }
        } else {
            let v: i32 = part.parse()?;
            if v >= min && v <= max {
                values.push(v);
            }
        }
    }
    if values.is_empty() {
        anyhow::bail!("cron field '{field}' resolved to no valid values");
    }
    Ok(values)
}

fn matches(value: i32, allowed: &[i32]) -> bool {
    allowed.is_empty() || allowed[0] == -1 || allowed.contains(&value)
}

fn month_matches(month: i32, allowed: &[i32]) -> bool {
    matches(month, allowed)
}

fn dom_matches(day: i32, allowed: &[i32]) -> bool {
    matches(day, allowed)
}

fn dow_matches(weekday: Weekday, allowed: &[i32]) -> bool {
    let num = weekday.num_days_from_sunday() as i32;
    matches(num, allowed) || matches(if num == 0 { 7 } else { num }, allowed)
}

fn hour_matches(hour: i32, allowed: &[i32]) -> bool {
    matches(hour, allowed)
}

fn minute_matches(minute: i32, allowed: &[i32]) -> bool {
    matches(minute, allowed)
}

fn advance_to_next_month(dt: DateTime<Utc>) -> DateTime<Utc> {
    let mut m = dt.month() + 1;
    let mut y = dt.year();
    if m > 12 {
        m = 1;
        y += 1;
    }
    NaiveTime::from_hms_opt(0, 0, 0)
        .and_then(|t| chrono::NaiveDate::from_ymd_opt(y, m, 1).map(|d| d.and_time(t)))
        .map(|d| DateTime::from_naive_utc_and_offset(d, Utc))
        .unwrap_or(dt + Duration::days(32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_cron() {
        let cron = interval_to_cron(&Interval::Daily);
        assert_eq!(cron, "0 2 * * *");
        let now = Utc::now();
        let next = next_run_from_cron(cron, &now).unwrap();
        assert!(next > now);
        assert_eq!(next.hour(), 2);
        assert_eq!(next.minute(), 0);
        assert_eq!(next.second(), 0);
    }

    #[test]
    fn test_hourly_cron() {
        let cron = interval_to_cron(&Interval::Hourly);
        assert_eq!(cron, "0 * * * *");
        let now = Utc::now();
        let next = next_run_from_cron(cron, &now).unwrap();
        assert!(next > now);
        assert_eq!(next.minute(), 0);
        assert_eq!(next.second(), 0);
    }
}
