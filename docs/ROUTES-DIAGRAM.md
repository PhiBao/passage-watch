# Route outcome diagram

This diagram explains the product's central distinction: the same holding can reach different end states through different processes. It is the single most useful diagram for a judge.

```text
                         +-------------------+
                         |  Exact holding    |
                         |  AAPLX / Solana   |
                         |  75 units         |
                         +---------+---------+
                                   |
          +------------------------+------------------------+
          |                        |                        |
          v                        v                        v
  +---------------+        +----------------+       +----------------+
  | Secondary     |        | Market Flow    |       | xChange        |
  | sale          |        | + xChange      |       | (atomic RFQ)   |
  |               |        |                |       |                |
  | -> USDC now   |        | -> stablecoin  |       | -> stablecoin  |
  |               |        |   proceeds     |       |   atomically   |
  | quote: live   |        |                |       |                |
  | fill: not     |        | issuer onboard |       | issuer onboard |
  | guaranteed    |        | + whitelisted  |       | + ~60s window  |
  +---------------+        +----------------+       +----------------+
          |                        |                        |
          +---------- DESTINATION: USDC IN WALLET ----------+
                                   |
             switching destination to shares changes the answer
                                   |
          +--------------------------------------------------+
          | xPort (in-kind)                                    |
          | -> underlying shares in Alpaca brokerage           |
          | requires issuer AND brokerage onboarding           |
          | shares are NOT cash; minimum not published         |
          +--------------------------------------------------+
```

## What to read from it

- Secondary sale is the only route with a live, size-specific quote in the MVP.
- Market Flow and xChange both target stablecoins but have different prerequisites and pricing models.
- xPort is not a variant of the others — it delivers a different asset (shares).
- No route is marked "best." The holder chooses the destination; the planner shows the consequences.

The interactive version of this is the destination switch in the app itself.
