# Passage Watch — build goal (Phase 0–2, in progress)

Build the next product version of Passage Watch: a **tokenized-stock exit readiness monitor**, not a one-time route planner.

## Phase 0 — Correctness (DONE, committed)
- Sell-side Jupiter quote corrected to **token → USDC** direction; live value verified ($26,536 sell of 75 AAPLX).
- Verified via Solana parsed-mint decimals + active multiplier (Solar Scaled UI rule).
- NYSE calendar for session display; server caches for CMC calls; clean tests.

## Non-negotiables

- **Correct sell-side quote:** the user enters "I hold N of token X; I want USDC." The quote must be token X → USDC, using exact base units with the active Token-2022 multiplier. No hardcoded reference prices in the quote path.
- **No fake live states:** quote failure, RPC failure, stale data → explicit "unknown / no quote / stale."
- **Destination-specific:** Secondary Sale vs Market Flow vs xChange (stablecoin), xPort (shares) — different outcomes, prerequisites, evidence.
- **Readiness model:** Quoted, Ready to confirm, Blocked by prerequisite, Terms only, Different destination, Unknown applicability, No quote from this provider, Stale evidence, Conflicting evidence.
- **No global "best route" verdict.** One result per route.

## Phase 0 — Correctness (priority)

1. Fix the sell quote: input mint = token, output = USDC, exact raw units = quantity × multiplier-scaled.
2. Use the user's *destination* semantics: USDC-in-wallet → sell quote; brokerage → show xPort + Market Flow as stablecoin-side exit analysis.
3. Remove hardcoded reference prices from the quote path.
4. Add a realistic US market session calendar (Weekdays 13:30–20:00 UTC; holidays).
5. Add server caches: CMC catalog/contract/multiplier, quote coalescing.
6. Round-trip tests proving the sell quote amount is sensible relative to an independent cross-check (compare JAX/outer spread vs observed price).
7. Real recorded-response tests for quote parsing.

## Phase 1 — Living CMC catalog

8. Sync CMC RWA map, quotes latest, issuers list/detail, crypto info.
9. Auto-discover Backed (xStocks) Solana tokens; enable instruments with verified identity + contract + unit.
10. Show covered count + unsupported reasons in the UI catalog.
11. Target ≥20 enabled instruments.

## Phase 2 — Readiness timeline + monitoring

13. Exit-window timeline: Now → Next US session → Your deadline → route windows.
14. Global snapshot worker (server, interval 15–30 min): CMC price/identity, Jupiter sell quote at standard sizes, session, active multiplier, evidence freshness, token/issuer graph changes.
15. SQLite observation store; saved plans are compared against real collected history.
16. Route-evidence change detection (source diff vs stored version).
17. UI: "Change since saved" from real snapshots (not just a re-check), plus readiness timeline.

## What NOT to do

- No AI chat.
- No execution / wallet signing / redemption.
- No generic RWA dashboard, security products, or "best route" winner.
- Don't add breadth (second issuer) until Phase 1 proves the catalog works.
- Don't expand past instruments that verify cleanly.

## Phase 1 implementation notes

Fetch: `GET /v5/real-world-assets/map` (symbol=AAPL,NVDA,TSLA,SPY + broader list),
`GET /v5/real-world-assets/quotes/latest` (per underlying rwa_id),
`GET /v5/real-world-assets/issuers/list` and `/issuers` (cross-check roster),
`GET /v2/cryptocurrency/info` (contracts).
Verification: identity + contract + unit + quote-support. 
Acceptance: catalog from CMC, not hardcoded; show "covered X of Y".
