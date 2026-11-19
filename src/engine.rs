//! Deterministic, destination-specific route evaluation.
//!
//! The engine consumes a fully supplied PlanContext and returns one RouteResult
//! per route. It deliberately never ranks routes or produces a "best" verdict,
//! because issuer quotes, personal access and actual proceeds are unavailable.

use crate::evidence::{self, RULE_VERSION};
use crate::model::{Destination, MarketSession, Onboarding, PlanContext};
use crate::route::{EvidenceState, RouteKind, RouteResult};

fn state_label(value: EvidenceState) -> &'static str {
    match value {
        EvidenceState::Quoted => "Quoted",
        EvidenceState::TermsOnly => "Terms only",
        EvidenceState::RequiresPreparation => "Requires preparation",
        EvidenceState::DifferentDestination => "Different destination",
        EvidenceState::UnknownApplicability => "Unknown applicability",
        EvidenceState::ConflictingEvidence => "Conflicting evidence",
        EvidenceState::NoQuote => "No quote from this provider",
        EvidenceState::Stale => "Stale",
    }
}

fn onboarding_label(value: Onboarding) -> &'static str {
    match value {
        Onboarding::Met => "reported met",
        Onboarding::Unmet => "reported not met",
        Onboarding::Unknown => "unknown",
    }
}

fn sources_for(reg: &[evidence::RouteEvidence], route: &str) -> Vec<String> {
    let mut urls: Vec<String> = reg
        .iter()
        .find(|e| e.route == route)
        .map(|e| e.sources.iter().map(|s| s.url.clone()).collect())
        .unwrap_or_default();
    urls.sort();
    urls.dedup();
    urls
}

pub fn evaluate(ctx: &PlanContext, today: crate::date::SimpleDate) -> Vec<RouteResult> {
    let reg = evidence::registry(today);
    let stale = reg
        .iter()
        .any(|e| e.freshness_days > 30 || e.changed_awaiting_review);
    let wants_usdc = ctx.destination == Destination::UsdcSolana;
    let mut out = Vec::new();

    // 1. Secondary sale — only route with a size-specific quote.
    let secondary_state = if ctx.quote_expired {
        EvidenceState::Stale
    } else if ctx.has_secondary_quote {
        EvidenceState::Quoted
    } else {
        EvidenceState::NoQuote
    };
    out.push(RouteResult {
        kind: RouteKind::SecondarySale,
        title: "Secondary sale".into(),
        outcome: format!(
            "Sell {} {} on a secondary venue for the destination token.",
            ctx.quantity, ctx.instrument.token_symbol
        ),
        applicable: wants_usdc,
        evidence_state: secondary_state,
        prerequisites: vec!["A supported trading venue and pair".into()],
        unknowns: vec![
            "No quote from this provider does not mean unsellable".into(),
            "Quoted output is an estimate, not a guaranteed fill".into(),
        ],
        source_urls: vec!["https://developers.jup.ag/docs/api-reference/swap/order.md".into()],
        when: Some(if ctx.has_secondary_quote {
            "Immediately (quote is live)".into()
        } else {
            "Whenever a quote is live (provider-dependent)".into()
        }),
    });

    // 2. Market Flow — stablecoin proceeds, whitelisted direct client.
    out.push(RouteResult {
        kind: RouteKind::MarketFlow,
        title: "Market Flow".into(),
        outcome: "Broker sale followed by stablecoin proceeds (USDC / USDG / USDT).".into(),
        applicable: wants_usdc,
        evidence_state: if stale {
            EvidenceState::Stale
        } else {
            EvidenceState::RequiresPreparation
        },
        prerequisites: vec![
            format!(
                "Issuer onboarding: {}",
                onboarding_label(ctx.issuer_onboarding)
            ),
            "Whitelisted direct-client wallet".into(),
        ],
        unknowns: vec![
            "$5,000 documented minimum; applies to brokerage onboarded direct clients".into(),
            "Published settlement figures are averages, not a deadline".into(),
            "Actual sale proceeds not known without operational access".into(),
        ],
        source_urls: sources_for(&reg, "MARKET_FLOW"),
        when: Some(when_for_session(ctx.market_session)),
    });

    // 3. xChange — atomic stablecoin settlement, onboarded clients.
    out.push(RouteResult {
        kind: RouteKind::Xchange,
        title: "xChange (atomic RFQ)".into(),
        outcome: "Atomic token/stablecoin settlement on supported EVM chains and Solana.".into(),
        applicable: wants_usdc,
        evidence_state: if stale {
            EvidenceState::Stale
        } else {
            EvidenceState::RequiresPreparation
        },
        prerequisites: vec![format!(
            "Issuer onboarding: {}",
            onboarding_label(ctx.issuer_onboarding)
        )],
        unknowns: vec![
            "Variable spread by trading period; no current quote without access".into(),
            "~60 second authorization window".into(),
            "Route minimum not specified on the route page — confirm with issuer".into(),
            "User access and availability not verified".into(),
        ],
        source_urls: sources_for(&reg, "XCHANGE"),
        when: Some(when_for_session(ctx.market_session)),
    });

    // 4. xPort — shares in brokerage, distinct destination.
    let xport_ready =
        ctx.issuer_onboarding == Onboarding::Met && ctx.broker_onboarding == Onboarding::Met;
    out.push(RouteResult {
        kind: RouteKind::Xport,
        title: "xPort (in-kind)".into(),
        outcome: "Underlying shares delivered to an Alpaca brokerage account.".into(),
        applicable: !wants_usdc,
        evidence_state: if !wants_usdc {
            if stale {
                EvidenceState::Stale
            } else {
                EvidenceState::RequiresPreparation
            }
        } else {
            EvidenceState::DifferentDestination
        },
        prerequisites: vec![
            format!(
                "Issuer onboarding: {}",
                onboarding_label(ctx.issuer_onboarding)
            ),
            format!(
                "Brokerage (Alpaca) onboarding: {}",
                onboarding_label(ctx.broker_onboarding)
            ),
        ],
        unknowns: vec![
            "Shares received are not cash received".into(),
            xport_ready_message(xport_ready),
            "Route minimum and settlement duration not published on the route page".into(),
        ],
        source_urls: sources_for(&reg, "XPORT"),
        when: Some("When both onboardings are met".into()),
    });

    if ctx.market_session == MarketSession::UsMarketClosed {
        for r in &mut out {
            if matches!(r.kind, RouteKind::MarketFlow | RouteKind::Xport) {
                r.unknowns
                    .push("Issuer routes operate 24/5; currently outside that window".into());
            }
        }
    }

    out
}

fn when_for_session(session: MarketSession) -> String {
    match session {
        MarketSession::UsMarketOpen => "Now".into(),
        MarketSession::UsMarketClosed => "When the US session opens".into(),
    }
}

fn xport_ready_message(ready: bool) -> String {
    if ready {
        "Both onboardings reported met; still not verified access".into()
    } else {
        "Issuer onboarding alone does not satisfy Alpaca onboarding".into()
    }
}

pub fn state_text(state: EvidenceState) -> &'static str {
    state_label(state)
}

pub fn rule_version() -> &'static str {
    RULE_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Instrument, UnitBasis};

    fn ctx(
        dest: Destination,
        issuer: Onboarding,
        broker: Onboarding,
        session: MarketSession,
    ) -> PlanContext {
        PlanContext {
            instrument: Instrument {
                underlying: "AAPL".into(),
                underlying_name: "Apple Inc.".into(),
                rwa_id: 3,
                token_symbol: "AAPLX".into(),
                crypto_id: 36994,
                issuer: "Backed Assets".into(),
                network: "Solana".into(),
                contract: "XsbEhLAtcf6HdfpFZ5xEMdqW8nfAvcsP5bdudRLJzJp".into(),
                observed_price_usd: Some(350.0),
                unit_basis: UnitBasis::Verified,
            },
            quantity: 75.0,
            destination: dest,
            deadline: "2026-11-20".into(),
            issuer_onboarding: issuer,
            broker_onboarding: broker,
            market_session: session,
            has_secondary_quote: true,
            quote_output_amount: Some(1000.0),
            quote_expired: false,
            evidence_reviewed_at: "2026-11-18".into(),
        }
    }

    #[test]
    fn xport_requires_both_onboardings() {
        let results = evaluate(
            &ctx(
                Destination::BrokerageShares,
                Onboarding::Met,
                Onboarding::Unknown,
                MarketSession::UsMarketOpen,
            ),
            crate::date::SimpleDate::new(2026, 11, 18),
        );
        let xport = results.iter().find(|r| r.kind == RouteKind::Xport).unwrap();
        assert!(xport.applicable);
        assert!(xport
            .unknowns
            .iter()
            .any(|u| u.contains("Issuer onboarding alone")));
    }

    #[test]
    fn destination_mismatch_yields_different_destination() {
        let results = evaluate(
            &ctx(
                Destination::UsdcSolana,
                Onboarding::Unknown,
                Onboarding::Unknown,
                MarketSession::UsMarketOpen,
            ),
            crate::date::SimpleDate::new(2026, 11, 18),
        );
        let xport = results.iter().find(|r| r.kind == RouteKind::Xport).unwrap();
        assert!(!xport.applicable);
        assert_eq!(xport.evidence_state, EvidenceState::DifferentDestination);
    }

    #[test]
    fn no_quote_never_means_unsellable() {
        let mut c = ctx(
            Destination::UsdcSolana,
            Onboarding::Unknown,
            Onboarding::Unknown,
            MarketSession::UsMarketOpen,
        );
        c.has_secondary_quote = false;
        let results = evaluate(&c, crate::date::SimpleDate::new(2026, 11, 18));
        let sec = results
            .iter()
            .find(|r| r.kind == RouteKind::SecondarySale)
            .unwrap();
        assert_eq!(sec.evidence_state, EvidenceState::NoQuote);
        assert!(sec
            .unknowns
            .iter()
            .any(|u| u.contains("does not mean unsellable")));
    }

    #[test]
    fn closed_market_adds_24_5_warning() {
        let results = evaluate(
            &ctx(
                Destination::UsdcSolana,
                Onboarding::Unknown,
                Onboarding::Unknown,
                MarketSession::UsMarketClosed,
            ),
            crate::date::SimpleDate::new(2026, 11, 18),
        );
        let mf = results
            .iter()
            .find(|r| r.kind == RouteKind::MarketFlow)
            .unwrap();
        assert!(mf.unknowns.iter().any(|u| u.contains("24/5")));
    }
}
