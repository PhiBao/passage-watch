//! A small US-market calendar used only for display of session state and the
//! exit-window timeline. It encodes the NYSE holiday set (2026–2027) and
//! weekend closure. We do not aim for full authority: it powers UX and
//! "ready-ish" messaging, and every claim about operator-time access still
//! routes through the primary issuer documents.

use std::collections::HashSet;

/// NYSE regular session: weekdays, 13:30–20:00 UTC (9:30 AM–4:00 PM ET).
const SESSION_START_HOUR_UTC: u32 = 13;
const SESSION_END_HOUR_UTC: u32 = 20;

fn holidays() -> &'static HashSet<(i32, u32, u32)> {
    static HOLIDAYS: &[(i32, u32, u32)] = &[
        (2026, 1, 1),
        (2026, 1, 19),
        (2026, 2, 16),
        (2026, 4, 3),
        (2026, 5, 25),
        (2026, 6, 19),
        (2026, 7, 3),
        (2026, 9, 7),
        (2026, 11, 26),
        (2026, 12, 25),
        (2027, 1, 1),
        (2027, 1, 18),
        (2027, 2, 15),
        (2027, 4, 2),
        (2027, 5, 24),
        (2027, 6, 18),
        (2027, 7, 4),
        (2027, 7, 3),
        (2027, 9, 6),
        (2027, 11, 25),
        (2027, 12, 24),
        (2027, 12, 25),
    ];
    static SET: std::sync::OnceLock<HashSet<(i32, u32, u32)>> = std::sync::OnceLock::new();
    SET.get_or_init(|| HOLIDAYS.iter().copied().collect())
}

pub fn is_usdst_holiday(year: i32, month: u32, day: u32) -> bool {
    holidays().contains(&(year, month, day))
}

/// Returns true if the given UTC datetime is a regular NYSE session.
pub fn session_is_open(year: i32, month: u32, day: u32, weekday: u32, hour: u32) -> bool {
    if !(1..=5).contains(&weekday) {
        return false;
    }
    if is_usdst_holiday(year, month, day) {
        return false;
    }
    hour >= SESSION_START_HOUR_UTC && hour < SESSION_END_HOUR_UTC
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_2026_new_year() {
        assert!(is_usdst_holiday(2026, 1, 1));
        assert!(!is_usdst_holiday(2026, 1, 5));
    }

    #[test]
    fn session_window_mid_day() {
        // Mid-day during a trading session.
        assert!(session_is_open(2026, 11, 18, 3, 14));
        // Closed before the session opens.
        assert!(!session_is_open(2026, 11, 18, 3, 11));
        // Closed on holiday (Dec 25).
        assert!(!session_is_open(2026, 12, 25, 5, 12));
    }
}
