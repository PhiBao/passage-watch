# Demo script (≈2:30)

Record this against the deployed app or `http://localhost:3000`. Everything shown is live data.

## 0:00–0:20 — Frame the decision

Open the app and let the status bar sit for a beat: the UTC clock ticks, the NYSE badge shows the live session state, and the tape carries every catalog instrument's current CMC price.

> "A tokenized stock is a wrapper. Before you act, two questions matter: what does it actually become — cash or shares — and can that happen before your deadline? Passage Watch answers that with live data."

## 0:20–0:50 — Pick the holding

- Instrument: **AAPLX — Apple Inc. (Solana)**.
- Quantity: **75**.
- Destination: **USDC in my Solana wallet**.
- Deadline: pick a date ~2 days out.
- Leave onboarding on **unknown**.
- Click **Check routes**.

Point at the verdict card:

> "75 AAPLX sells for about $26.5k of USDC through a read-only Jupiter quote — here's the per-share price next to CMC's observed price, so you can see the spread."

## 0:50–1:15 — Liquidity at your size

Scroll to **Liquidity at your size**.

> "Same quote at 25, 50 and 100 percent of the position. At this liquidity the price barely moves — that's the honest answer for this size."

## 1:15–1:40 — The routes are different

> "Selling for USDC is one route. Market Flow and xChange are issuer routes — they need onboarding and they open with the US session. And xPort delivers shares, not cash."

Point at each card's **Available:** line and **Unknowns**.

## 1:40–2:00 — The exit window

Scroll to **Exit window**.

> "The market is closed now. The next session opens at 13:00 UTC, and there are two sessions before the deadline. That's the real constraint — not a countdown, a calendar."

## 2:00–2:20 — Watch it

- Click **Watch this plan**.
- Change the quantity slightly or re-check after a few seconds.
- Show **Changes since saved** and the sparkline.

> "The plan is saved locally; the app remembers the observation and tells you what moved."

## 2:20–2:30 — Honesty close

> "Quotes are estimates, not fills. Shares aren't cash. Self-reported onboarding is never verified eligibility. CMC supplies the identity graph and observed prices; route terms are separately sourced and dated."

## Browser fallback

If the deployed link is unavailable, record the same flow locally:

```bash
cp .env.example .env   # add the campaign key
cargo run              # serves API + built frontend on :3000
```

Then open `http://localhost:3000`.
