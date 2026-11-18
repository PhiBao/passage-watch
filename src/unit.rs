//! Active-multiplier selection for Solana Token-2022 Scaled UI Amount.
//!
//! We read the *parsed* mint via RPC (`getAccountInfo` with jsonParsed), which
//! returns clean strings and the effective timestamp — no hand-rolled TLV parse.
//! The active multiplier is the new one only once its effective timestamp has
//! passed; otherwise the old one remains active. If anything is ambiguous, the
//! unit basis is UNKNOWN and the caller must suppress valuation.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaledUiConfig {
    pub multiplier: f64,
    pub new_multiplier: f64,
    pub new_multiplier_effective_timestamp: i64,
}

/// Selects the multiplier that is active at `now_unix`.
///
/// Rules:
/// - If there is no pending change (timestamps equal / new <= old), use `multiplier`.
/// - Otherwise use `new_multiplier` only when `effective <= now`, else `multiplier`.
/// - Invalid/non-finite values return None (unknown unit).
pub fn active_multiplier(cfg: &ScaledUiConfig, now_unix: i64) -> Option<f64> {
    let valid = cfg.multiplier.is_finite()
        && cfg.multiplier > 0.0
        && cfg.new_multiplier.is_finite()
        && cfg.new_multiplier > 0.0;
    if !valid {
        return None;
    }
    let pending = cfg.new_multiplier_effective_timestamp > 0;
    if pending && cfg.new_multiplier_effective_timestamp <= now_unix {
        Some(cfg.new_multiplier)
    } else {
        Some(cfg.multiplier)
    }
}

/// Converts a displayed (scaled) quantity into raw base units, rounding down.
/// Returns None when the unit basis is unresolved.
pub fn displayed_to_base(displayed: f64, decimals: u8, multiplier: f64) -> Option<u128> {
    if !displayed.is_finite() || displayed < 0.0 || !multiplier.is_finite() || multiplier <= 0.0 {
        return None;
    }
    let raw = (displayed * 10_f64.powi(decimals as i32)) / multiplier;
    if raw < 0.0 || raw > u128::MAX as f64 {
        return None;
    }
    Some(raw.floor() as u128)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(eff: i64) -> ScaledUiConfig {
        ScaledUiConfig {
            multiplier: 1.0032690125398187,
            new_multiplier: 1.0038023030588032,
            new_multiplier_effective_timestamp: eff,
        }
    }

    #[test]
    fn picks_new_only_after_effective_time() {
        let before = active_multiplier(&cfg(1794447000), 1794446999).unwrap();
        let after = active_multiplier(&cfg(1794447000), 1794447000).unwrap();
        assert_eq!(before, 1.0032690125398187);
        assert_eq!(after, 1.0038023030588032);
    }

    #[test]
    fn no_pending_change_uses_old() {
        assert_eq!(
            active_multiplier(&cfg(0), 2_000_000_000).unwrap(),
            1.0032690125398187
        );
    }

    #[test]
    fn invalid_config_is_unknown() {
        let bad = ScaledUiConfig {
            multiplier: f64::NAN,
            new_multiplier: 1.0,
            new_multiplier_effective_timestamp: 1,
        };
        assert!(active_multiplier(&bad, 2_000_000_000).is_none());
    }

    #[test]
    fn converts_displayed_to_base_rounding_down() {
        // 2 share-equivalents, 8 decimals, multiplier 2 -> 1 raw unit per share.
        let base = displayed_to_base(2.0, 8, 2.0).unwrap();
        assert_eq!(base, 100_000_000);
    }
}
