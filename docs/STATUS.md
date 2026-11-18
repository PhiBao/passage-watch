# Status

## Current phase

Scaffold and first vertical slice.

## Done

- Product direction fixed: Passage Watch.
- Repository safety boundaries added.
- CMC key stored only in `.env`, which is ignored by Git.

## Next

- Verify the campaign key's real entitlements and selected token coverage.
- Build the Rust API shell and React shell.
- Implement exact CMC identity resolution and a real-call evidence path.
- Implement route rules and destination comparison.
- Implement local plan storage and change detection.

## Known risks

- Actual event-key endpoint entitlement is unverified.
- RWA and crypto market-pairs endpoints are Growth-gated in current references; Passage must run on Basic after the event.
- Secondary quote provider coverage and display terms are unverified.
- Token unit semantics require effective-time multiplier handling.
- Issuer route evidence can become stale.
- The product remains adjacent to strong prior projects; monitoring and destination semantics must remain central.
