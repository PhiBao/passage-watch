# Passage Watch — build goal

Build a submission-ready, read-only Rust + React prototype in this repository.

## Objective

Passage Watch helps a self-custody xStock holder decide how to turn a specific tokenized holding into a chosen destination (Solana USDC or brokerage shares) by a deadline. It must show the relevant routes, prerequisites, route-level unknowns, market/quote evidence, and what changed since the plan was saved.

It is explicitly **not** an execution product, a legal-eligibility engine, or a universal RWA dashboard.

## Required end state

- Public, submission-quality GitHub-ready repository.
- Rust backend with the CMC key server-side only.
- React/TypeScript frontend with the Passage visual identity.
- Real CMC endpoint integration and real API-call evidence.
- Exact token identity and unit handling.
- Destination-specific comparison: secondary sale, Market Flow, xChange, xPort.
- Local saved plan and change-since-saved behavior.
- Read-only secondary quote adapter; quote never presented as a fill.
- Explicit evidence, freshness, conflict, stale, unavailable, and unknown states.
- Basic-tier-compatible fallback behavior.
- Tests, README, architecture/route diagrams, favicon and design tokens.
- A judge-first 60-second path and a short demo script.

## Non-negotiables

- Never expose `CMC_API_KEY` to the browser, logs, tests, fixtures, or Git.
- Do not claim legal eligibility, NAV, guaranteed redemption, or guaranteed execution.
- Do not include synthetic API responses presented as live evidence.
- Do not add wallet signing, redemption, or trading execution.
- Do not ship a generic dashboard merely to use more endpoints.

## Loop behavior

After each implementation pass:

1. Run the smallest relevant test/build/typecheck.
2. Fix all failures in that area.
3. Inspect the change for the non-negotiables.
4. Record the remaining risk in `docs/STATUS.md`.
5. Continue to the next vertical slice until the objective is met or a hard blocker is proven.

If a dependency cannot be verified, implement the honest degraded state rather than fabricating a result.
