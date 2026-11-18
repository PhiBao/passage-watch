//! A minimal, dependency-free calendar date used for deterministic evidence aging.

/// A valid proleptic Gregorian calendar date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimpleDate {
    year: i32,
    month: u32,
    day: u32,
}

impl SimpleDate {
    pub const fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }

    /// Parses `YYYY-MM-DD`. Returns `None` for malformed or impossible dates.
    pub fn parse_iso(text: &str) -> Option<Self> {
        let mut parts = text.split('-');
        let year = parts.next()?.parse::<i32>().ok()?;
        let month = parts.next()?.parse::<u32>().ok()?;
        let day = parts.next()?.parse::<u32>().ok()?;
        if parts.next().is_some() {
            return None;
        }
        let date = Self::new(year, month, day);
        if date.is_valid() {
            Some(date)
        } else {
            None
        }
    }

    pub fn to_iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    pub fn is_valid(&self) -> bool {
        if self.year < 1 || !(1..=12).contains(&self.month) || self.day < 1 {
            return false;
        }
        self.day <= days_in_month(self.year, self.month)
    }

    /// Whole days from `earlier` to `self`. Negative when `earlier` is later.
    pub fn days_since(&self, earlier: &Self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
            - days_from_civil(earlier.year, earlier.month, earlier.day)
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Days since 1970-01-01 (Howard Hinnant's `days_from_civil`).
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y = year as i64;
    let m = month as i64;
    let d = day as i64;
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_formats_iso_dates() {
        let date = SimpleDate::parse_iso("2025-05-01").unwrap();
        assert_eq!(date, SimpleDate::new(2025, 5, 1));
        assert_eq!(date.to_iso(), "2025-05-01");
        assert!(SimpleDate::parse_iso("2025-02-30").is_none());
        assert!(SimpleDate::parse_iso("not-a-date").is_none());
        assert!(SimpleDate::parse_iso("2025-5-1").is_some());
    }

    #[test]
    fn counts_days_across_months_and_leap_years() {
        assert_eq!(
            SimpleDate::new(2025, 5, 1).days_since(&SimpleDate::new(2025, 4, 15)),
            16
        );
        assert_eq!(
            SimpleDate::new(2024, 3, 1).days_since(&SimpleDate::new(2024, 2, 28)),
            2
        );
        assert_eq!(
            SimpleDate::new(2025, 1, 1).days_since(&SimpleDate::new(2024, 1, 1)),
            366
        );
    }
}
