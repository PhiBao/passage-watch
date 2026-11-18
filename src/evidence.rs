//! Versioned, field-level route evidence. No legal conclusions — only what a
//! primary source documents, when it was reviewed, and what remains unknown.

use serde::{Deserialize, Serialize};

use crate::date::SimpleDate;

pub const RULE_VERSION: &str = "routes-2026-11-18";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRecord {
    pub field: String,
    pub value: String,
    pub url: String,
    pub document_date: String,
    pub reviewed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEvidence {
    pub route: &'static str,
    pub sources: Vec<SourceRecord>,
    pub freshness_days: i64,
    pub changed_awaiting_review: bool,
}

const XSTOCK_DOCS: &str = "https://docs.xstocks.fi/docs/issuance-and-redemption";

fn reviewed(date: &str, field: &str, value: &str, url: String, doc: &str) -> SourceRecord {
    SourceRecord {
        field: field.to_string(),
        value: value.to_string(),
        url,
        document_date: doc.to_string(),
        reviewed_at: date.to_string(),
    }
}

/// Builds the curated evidence registry for the documented xStocks routes.
/// `today` drives deterministic freshness aging for tests and the UI.
pub fn registry(today: SimpleDate) -> Vec<RouteEvidence> {
    let reviewed_on = "2026-11-18";
    let reviewed_date = SimpleDate::parse_iso(reviewed_on).unwrap();
    let age = today.days_since(&reviewed_date).max(0);

    vec![
        RouteEvidence {
            route: "MARKET_FLOW",
            sources: vec![
                reviewed(
                    reviewed_on,
                    "outcome",
                    "Broker sale, stablecoin proceeds",
                    format!("{}/market-flow", XSTOCK_DOCS),
                    "2026-10-22",
                ),
                reviewed(
                    reviewed_on,
                    "minimum",
                    "$5,000",
                    format!("{}/market-flow", XSTOCK_DOCS),
                    "2026-10-22",
                ),
                reviewed(
                    reviewed_on,
                    "availability",
                    "24/5, US market hours",
                    format!("{}/market-flow", XSTOCK_DOCS),
                    "2026-10-22",
                ),
                reviewed(
                    reviewed_on,
                    "fees",
                    "Currently set to 0",
                    format!("{}/market-flow", XSTOCK_DOCS),
                    "2026-10-22",
                ),
            ],
            freshness_days: age,
            changed_awaiting_review: false,
        },
        RouteEvidence {
            route: "XCHANGE",
            sources: vec![
                reviewed(
                    reviewed_on,
                    "outcome",
                    "Atomic token/stablecoin settlement",
                    format!("{}/atomic-rfq-xchange", XSTOCK_DOCS),
                    "2026-11-06",
                ),
                reviewed(
                    reviewed_on,
                    "pricing",
                    "Quote includes variable spread",
                    format!("{}/atomic-rfq-xchange", XSTOCK_DOCS),
                    "2026-11-06",
                ),
                reviewed(
                    reviewed_on,
                    "window",
                    "~60s authorization window",
                    format!("{}/atomic-rfq-xchange", XSTOCK_DOCS),
                    "2026-11-06",
                ),
                reviewed(
                    reviewed_on,
                    "minimum",
                    "Not specified on route page",
                    format!("{}/atomic-rfq-xchange", XSTOCK_DOCS),
                    "2026-11-06",
                ),
            ],
            freshness_days: age,
            changed_awaiting_review: false,
        },
        RouteEvidence {
            route: "XPORT",
            sources: vec![
                reviewed(
                    reviewed_on,
                    "outcome",
                    "Underlying shares in Alpaca brokerage",
                    format!("{}/in-kind-flow-xport", XSTOCK_DOCS),
                    "2026-10-22",
                ),
                reviewed(
                    reviewed_on,
                    "prerequisite",
                    "Issuer AND Alpaca onboarding",
                    format!("{}/in-kind-flow-xport", XSTOCK_DOCS),
                    "2026-10-22",
                ),
                reviewed(
                    reviewed_on,
                    "availability",
                    "24/5, US market hours",
                    format!("{}/in-kind-flow-xport", XSTOCK_DOCS),
                    "2026-10-22",
                ),
            ],
            freshness_days: age,
            changed_awaiting_review: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freshness_ages_from_review_date() {
        let same = registry(SimpleDate::new(2026, 11, 18));
        assert!(same.iter().all(|e| e.freshness_days == 0));
        let later = registry(SimpleDate::new(2026, 11, 28));
        assert!(later.iter().all(|e| e.freshness_days == 10));
    }

    #[test]
    fn xchange_minimum_is_explicitly_unknown() {
        let reg = registry(SimpleDate::new(2026, 11, 18));
        let xc = reg.iter().find(|e| e.route == "XCHANGE").unwrap();
        let min = xc.sources.iter().find(|s| s.field == "minimum").unwrap();
        assert_eq!(min.value, "Not specified on route page");
    }
}
