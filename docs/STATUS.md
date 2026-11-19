# Status

## Current phase

Phase 0–2 delivered: correct sell-side Jupiter quote, living CMC catalog (146 Backed instruments), NYSE calendar, server caches, readiness timeline, and a background monitor worker.

## Done

- **Phase 0 — Correctness:** sell-side Jupiter quote now correctly sells tokens → USDC (verified live: $26,543 USDC for 75 AAPLX). Correct Solana scaled-UI multiplier/decimals handling via `jsonParsed` RPC. NYSE calendar for session display; server caches for CMC catalog/contract/multiplier and Jupiter quotes. `src/unit.rs` conversion round-trips.
- **Phase 1 — Living CMC catalog:** 146 Backed instruments discovered from `/v5/real-world-assets/map` + `quotes/latest`. Contracts resolved per-token via `/v2/cryptocurrency/info`. Contracts + unit verified at `check` time.
- **Phase 2 — Timeline + monitoring:** `/api/v1/timeline` shows market status, next session, deadline, and route statuses (SECONDARY_SALE: quoted). Background worker snapshots price + standard-size sell quote every 60s into an in-memory ring; `/api/v1/changes/:symbol` exposes changes since a saved time. Verified live: changes for AAPLX; check = $26,543.
- **Phase 3 — User-input improvements:** Read-only address scan (`GET /api/v1/address/:wallet`) returns Backed tokens the wallet holds with balances (no signing). Frontend "Import holding" prefills from your wallet. Verdict panel shows what the plan tells you. Catalog coverage count in results.
- **Phase 3b — strength:** liquidity-at-size quotes (25/50/100% of holding), SVG price sparkline from real CMC daily quotes, and multi-holding address import (chips to pick a holding).
- Deterministic route engine with explicit states (no "best route" verdict).
- Versioned evidence registry with freshness/staleness.
- Frontend: plan form, destination switch, route cards, timeline panel, changes panel, evidence panel, sparkline, size-liquidity table, localStorage saved-plan diff, verdict card, address import.

## Verified end-to-end

- `GET /` serves the built frontend; favicon loads.
- `GET /api/v1/check` (AAPLX, USDC) → `unit_basis: verified`, `usdc_amount ≈ 26,543`, spreads vs CMC visible.
- `GET /api/v1/check` (AAPLX, brokerage) → xPort `requires_preparation`.
- `GET /api/v1/timeline` → `market_open_now: false`, `next_session`, `routes`.
- `GET /api/v1/changes/AAPLX` → monitor snapshots.
- `GET /api/v1/observations/36994` → 10 daily points.
- `cargo test` 20/20, `pnpm build` clean, zero warnings.

## Next (remaining for submission)

- Note: monitor ring is in-memory (300 snapshots/global); acceptable for MVP but consider moving to SQLite later.
- Record the 3-minute demo video (port 3317 locally, or the deployed URL).
- Confirm Jupiter keyless display terms and rate limits.
- Final submission rehearsal: public repo, DoraHacks build, X post with `#BuildwithCMC`.
