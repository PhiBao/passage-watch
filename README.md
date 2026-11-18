# Passage Watch

**Choose what your tokenized holding becomes — and know when the route is ready.**

Passage Watch is a read-only decision workspace for self-custody tokenized-stock holders who must choose between stablecoin liquidity and brokerage shares before a deadline. It compares relevant routes, outstanding preparation, market evidence, and changes since the holder saved a plan.

It does not trade, sign, redeem, determine legal eligibility, or claim guaranteed settlement.

## Status

Implementation in progress. See [docs/STATUS.md](docs/STATUS.md) for the current build state and open risks.

## Safety and source boundaries

| Layer | What it means |
|---|---|
| Observed market data | CoinMarketCap identity, price and timestamp evidence |
| Provider quote | A size-specific secondary-sale estimate, not a fill |
| Documented route terms | Published process and conditions, not verified personal access |

A quote is never cash received. Brokerage shares are not bank cash. Unknown fields stay unknown.

## Planned product flow

1. Identify the exact tokenized instrument and network.
2. Enter the displayed holding quantity.
3. Choose USDC in a Solana wallet or shares in a brokerage.
4. Set the desired deadline.
5. Compare relevant routes and unresolved prerequisites.
6. Watch the saved local plan for quote, session, identity, unit and evidence changes.
7. Export a preparation packet with sources and explicit unknowns.

## Local development

```bash
cp .env.example .env
# put the participant's own campaign key in .env
pnpm install
pnpm dev
```

The application key is server-only. Never commit `.env` or a captured response containing it.

## Documentation

- [Product and build status](docs/STATUS.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Design system](docs/DESIGN.md)
- [Submission checklist](docs/SUBMISSION.md)

## Licence

Implementation code will carry an open-source licence. Third-party data and route documentation remain subject to their respective terms.
