# Architecture

Passage Watch keeps the holder's plan local and the market observations global.

```text
Browser
  ├── plan inputs in localStorage
  ├── route comparison
  └── change-since-saved diff
        │
        ▼
Rust service (Axum)
  ├── CMC identity/price client
  ├── quote adapter (read-only)
  ├── unit and route engine
  ├── session/calendar state
  └── global observation store
```

## Source boundaries

| Source | Used for | Never interpreted as |
|---|---|---|
| CoinMarketCap | Asset → token → issuer identity, observed prices and timestamps | Legal rights, NAV, executable liquidity or eligibility |
| Issuer documents | Published route outcomes, prerequisites and current stated conditions | Verification of this user's account access |
| Secondary quote provider | Size-specific estimate | A guaranteed fill or cash received |

## Security rules

- The CMC key is read server-side from `.env`.
- Browser requests contain only product inputs.
- No upstream request URLs are accepted from clients.
- No API key, private key, wallet signature or user holdings are logged.
- The plan's financial inputs remain in the browser by default.

## Failure behavior

A failed quote remains “no quote from this provider.” It must never become “unsellable.” A stale issuer document remains “changed — awaiting review.” A missing unit basis suppresses valuation instead of guessing.

## Deployment target

One Linux service serves the built frontend and the API on the same origin. No serverless secrets, database service or message queue are required for the MVP.
