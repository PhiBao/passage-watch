# Status

## Current phase

Vertical slice complete and running end-to-end against live APIs. Ready for demo polish and submission rehearsal.

## Done

- Rust backend (Axum) + React frontend (Vite) both build; 14 Rust tests pass.
- Live CMC catalog, check, instrument, method and status endpoints.
- Live Solana parsed-mint multiplier resolution; unit_basis verified end-to-end.
- Live keyless Jupiter read-only quote, normalized to share-equivalents.
- Deterministic route engine: destination-specific, explicit unknowns, no global verdict.
- Versioned evidence registry with freshness and conflict states.
- Frontend: plan form, destination switch, route cards, evidence panel, localStorage plan + change view.
- Real-call evidence captured under `docs/evidence/`; reproduction script added.
- Submission-quality README, architecture/design/submission docs and route diagram added.

- Product direction fixed: Passage Watch.
- Repository safety boundaries added; `.env` ignored by Git.
- **CMC key verified live**: the `.env` key is a valid Startup plan (450,000 credits/month, 600 req/min, HTTP 200 on `/v1/key/info`).
  - An earlier "plan expired" result came from a stale shell variable with a different key; `.env` is the source of truth.
  - Backend must load `.env` with `dotenvy::dotenv_override()` so the stale shell var cannot win.
- **Real RWA data verified**: underlying stock symbols resolve to `rwa_id`, and `quotes/latest` returns the token → issuer → `crypto_id` graph with prices.
  - AAPLX (`rwa_id` 3 / `crypto_id` 36994), NVDAX (2 / 36992), TSLAX (14 / 37004), SPYX (86 / 37006), all Backed Assets.
  - Real response sample saved under `docs/evidence/`.
- **Solana contract addresses** captured from `/v2/cryptocurrency/info`.
- **Secondary quote verified keyless**: Jupiter Swap V2 `/order` returns a read-only quote (no `taker`) with no API key. This removes an access dependency.

- Observation history panel from real CMC daily quotes (10 points), powering the "change since saved" demo.
- Quote normalization uses the parsed mint's real `decimals` (not hardcoded 8) via a round-trip-tested pair of converters in `src/unit.rs`. 15/15 tests pass.

## Next (remaining for submission)

- Record the 3-minute demo video from the running app.
- Confirm Jupiter keyless display terms and rate limits; add a documented fallback state.
- Add a short quota/caching guard for repeated demo runs.
- Final submission rehearsal: public repo, DoraHacks build, X post with `#BuildwithCMC`.

## Known risks

- RWA and crypto market-pairs endpoints are Growth-gated; Passage runs on Basic after the event and does not depend on them.
- Jupiter display terms and rate limits for a public demo still need confirmation; the keyless quote is unauthenticated and shared-rate-limited.
- Solana RPC for mint/multiplier reading needs a provider; public endpoint may rate-limit.
- Token unit semantics require effective-time multiplier handling.
- Issuer route evidence can become stale.
- The product remains adjacent to strong prior projects; monitoring and destination semantics must remain central.
