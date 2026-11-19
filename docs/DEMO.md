# Demo script (≈2:00)

Record this against the deployed app or `http://localhost:3000`. Everything shown is live data.

The flow is deliberately one take for the USDC destination — the quote, the verdict, the liquidity ladder, the routes and the exit window are all part of the same check, so they are shown without re-entering the form. The **only** second flow is the brokerage destination, because that is the only thing that actually changes the answer.

## 0:00–0:20 — Frame the decision

Open the app and let the status bar sit for a beat: the UTC clock ticks, the NYSE badge shows the live session state, and the tape carries every catalog instrument's current CMC price.

> "A tokenized stock is a wrapper. Before you act, two questions matter: what does it actually become — cash or shares — and can that happen before your deadline? Passage Watch answers that with live data."

## 0:20–0:45 — Pick the holding

- Instrument: **AAPLX — Apple Inc. (Solana)**.
- Quantity: **75**.
- Destination: **USDC in my Solana wallet**.
- Deadline: pick a date ~2 days out.
- Leave onboarding on **unknown**.

> "The desk opens with the market already on it. The app asks for the things that decide the answer: the exact token, the size you hold, what it should become, and the date that matters."

## 0:45–1:15 — One take through the USDC check

Click **Check routes** once. Do not re-enter the form; scroll through the answer instead, in this order:

1. **Quote board** — hold on the big number.

   > "One click pulls a live quote for that exact size: what the sale comes back with, per share, and how far that sits from the market price. The quote is an estimate — never a guaranteed fill."

2. **Verdict** — the line above the quote.

   > "The verdict names what is still missing before you act."

3. **Liquidity ladder** (25/50/100%) — lean in; this is the punch-in moment.

   > "The same holding is priced at a quarter, at half, and at the full size, so you can see whether your own order is what moves the price."

4. **Routes**.

   > "Every route that applies to this destination is listed with its own prerequisites and unknowns — no single best route, just the ones that fit."

5. **Exit window** — end the take here.

   > "Then the plan is checked against the calendar: the next session, how many sessions are left before your date, and which routes can run in each window. That is the part a price chart cannot tell you."

## 1:15–1:35 — The brokerage beat (the only other flow)

Switch the destination to **Shares in my brokerage** and click **Check routes** again — this is the only repeat of the flow, and it is the point: the answer changes.

> "Now change the destination to shares instead of cash. The route that matters changes too: xPort now applies, and it names what it needs — both the issuer and a brokerage account. The quote board steps aside, because a share is not a stablecoin, and the app keeps the two exits separate on purpose."

## 1:35–1:50 — Optional: watch it

- Click **Watch this plan**, then re-run a check after a few seconds.
- Show **Changes since saved** and the sparkline.

> "The plan is saved locally; the app remembers the observation and tells you what moved."

## 1:50–2:00 — Honesty close

> "Quotes are estimates, not fills. Shares aren't cash. Self-reported onboarding is never verified eligibility. CMC supplies the identity graph and observed prices; route terms are separately sourced and dated."

## Browser fallback

If the deployed link is unavailable, record the same flow locally:

```bash
cp .env.example .env   # add the campaign key
cargo run              # serves API + built frontend on :3000
pnpm build             # or use the dev server below
```

Then open `http://localhost:3000`.
