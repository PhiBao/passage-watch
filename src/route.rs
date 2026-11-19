//! Route results and evidence states. One result per route — never a global verdict.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RouteKind {
    SecondarySale,
    MarketFlow,
    Xchange,
    Xport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceState {
    Quoted,
    TermsOnly,
    RequiresPreparation,
    DifferentDestination,
    UnknownApplicability,
    ConflictingEvidence,
    NoQuote,
    Stale,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResult {
    pub kind: RouteKind,
    pub title: String,
    pub outcome: String,
    pub applicable: bool,
    pub evidence_state: EvidenceState,
    pub prerequisites: Vec<String>,
    pub unknowns: Vec<String>,
    pub source_urls: Vec<String>,
    /// When this route becomes available to the user (display label), if known.
    pub when: Option<String>,
}
