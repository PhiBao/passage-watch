//! Data-source-neutral models for a single exit-planning check.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Destination {
    UsdcSolana,
    BrokerageShares,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Onboarding {
    Met,
    Unmet,
    Unknown,
}

impl Default for Onboarding {
    fn default() -> Self {
        Onboarding::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instrument {
    pub underlying: String,
    pub underlying_name: String,
    pub rwa_id: u64,
    pub token_symbol: String,
    pub crypto_id: u64,
    pub issuer: String,
    pub network: String,
    pub contract: String,
    pub observed_price_usd: Option<f64>,
    pub unit_basis: UnitBasis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UnitBasis {
    Verified,
    Unknown,
}

/// Everything the engine needs, supplied by the caller (the HTTP layer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanContext {
    pub instrument: Instrument,
    pub quantity: f64,
    pub destination: Destination,
    pub deadline: String,
    pub issuer_onboarding: Onboarding,
    pub broker_onboarding: Onboarding,
    pub market_session: MarketSession,
    pub has_secondary_quote: bool,
    pub quote_output_amount: Option<f64>,
    pub quote_expired: bool,
    pub evidence_reviewed_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MarketSession {
    UsMarketOpen,
    UsMarketClosed,
}

impl MarketSession {
    pub fn from_utc_hour(hour: u32) -> Self {
        // US regular session 13:30–20:00 UTC. Approximation is intentionally
        // labelled and never used to promise settlement.
        if (13..20).contains(&hour) {
            MarketSession::UsMarketOpen
        } else {
            MarketSession::UsMarketClosed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_onboarding_is_unknown() {
        assert_eq!(Onboarding::default(), Onboarding::Unknown);
    }

    #[test]
    fn market_session_from_hour() {
        assert_eq!(
            MarketSession::from_utc_hour(15),
            MarketSession::UsMarketOpen
        );
        assert_eq!(
            MarketSession::from_utc_hour(2),
            MarketSession::UsMarketClosed
        );
    }
}
