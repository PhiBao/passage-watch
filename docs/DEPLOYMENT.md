# Deployment

The default, submission-safe path: deploy the single Rust service — it serves the production frontend from `dist/`. This is the target used to produce a reachable demo link.

## Option A — Docker image (recommended)

```bash
# Build the image (needs the participant's campaign key in .env at build time)
docker build -t passage-watch .

# Run it
docker run -d --name passage-watch -p 3000:3000 passage-watch
```

The container runs `entrypoint.sh`, which unsets any stale `CMC_API_KEY` shell variable so the `.env` file inside the repo is authoritative. To build without a key locally, set `PORT` and run `cargo build --release` directly.

To expose a public URL, point your usual ingress/tunnel (e.g. `docker port`, ngrok, or a platform's redeploy from the image) at container port `3000`.

## Option B — Local development

```bash
cp .env.example .env   # fill in the campaign key
cargo run              # serves API + static frontend on :3000
# or, in a second terminal: pnpm dev   # frontend on :5173, proxies /api to :3000
```

## Required environment

| Variable | Source | Notes |
|---|---|---|
| `CMC_API_KEY` | `.env` | Participant's campaign key. Never commit. |
| `PORT` | optional | Defaults to `3000`. |

The product does not read any other secrets or environment at runtime; the browser sends only product inputs.

## Notes for judges

- The API is same-origin (`/api/v1/*`) and the frontend is served by the same process in production.
- All upstream hosts are fixed constants in code (`pro-api.coinmarketcap.com`, `api.mainnet-beta.solana.com`, Jupiter); no user-supplied URLs.
- The key is never in browser bundles, logs, or `dist/`.
