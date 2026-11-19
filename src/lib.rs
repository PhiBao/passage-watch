//! Passage Watch planning engine and route-evidence registry.
//!
//! This crate is deliberately data-source-neutral: it performs no network
//! calls and has no external dependencies. It evaluates destination-specific
//! routes against a fully supplied [`model::PlanContext`] and ages the
//! versioned issuer evidence registered in [`evidence`].
//!
//! The engine never produces a global "best route" verdict. It returns one
//! [`route::RouteResult`] per route kind, each carrying its own outcome,
//! applicability, prerequisites, evidence state and explicit unknowns.

pub mod cache;
pub mod calendar;
pub mod cmc;
pub mod config;
pub mod date;
pub mod engine;
pub mod evidence;
pub mod monitor;
pub mod model;
pub mod route;
pub mod rpc;
pub mod sell_quote;
pub mod unit;

pub use date::SimpleDate;
