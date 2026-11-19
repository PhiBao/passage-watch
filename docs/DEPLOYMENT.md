# Deployment

Two supported paths. Both serve the API and the built frontend from the same
Rust process (`dist/` is produced by `pnpm build`).

## Primary: Fly.io, scale-to-zero

The live configuration is `fly.toml`:

| Setting | Value | Why |
|---|---|---|
| `min_machines_running` | `0` | No idle cost |
| `auto_stop_machines` | `stop` | Machine fully stops when idle |
| `auto_start_machines` | `true` | First request wakes it |
| VM | `shared-cpu-1x`, 512 MB | Smallest that runs the service comfortably |
| Health check | `GET /health` every 15s | Fast, cheap readiness |

Cold start is a few seconds. Fly's proxy holds the incoming request while the
machine boots, so the browser receives the page instead of an error.
`index.html` renders a small "starting the workspace" state until React mounts.

### First deploy

```bash
fly apps create passage-watch
fly secrets set CMC_API_KEY="<campaign key>" -a passage-watch
fly deploy --remote-only --ha=false -a passage-watch
```

`--ha=false` keeps a single machine, which is what scale-to-zero wants.

### Rotate the key

```bash
fly secrets set CMC_API_KEY="<new key>" -a passage-watch
```

No code change or redeploy is needed; the machine restarts with the new value.

## Alternative: local Docker

```bash
docker build -t passage-watch .
docker run -d --name passage-watch -p 3000:3000 \
  -e CMC_API_KEY="<campaign key>" passage-watch
```

The image has no `.env`; the key comes from the environment. `dotenvy` only
overrides when a `.env` file is present (local development).

## Environment

| Variable | Required | Notes |
|---|---|---|
| `CMC_API_KEY` | yes | Campaign key / Fly secret. Never committed. |
| `PORT` | no | Defaults to `3000`. |
| `ENABLE_MONITOR` | no | `1` enables the optional background snapshot worker. Off by default: checks record snapshots on demand instead, which keeps CMC quota low on the free tier. |

## Cost notes

- CMC calls are cached (`src/cache.rs`): catalog/contract/multiplier ~45s.
- The deployed app does **not** poll in the background by default. Each route
  check performs a handful of CMC calls plus one Jupiter sell quote per size.
- After the hackathon the key reverts to Basic (15,000 credits/month). The app
  is designed around that: on-check snapshots replace the polling worker.

## Judge checklist

- `GET /health` returns `ok`.
- `GET /` returns the app shell; the boot state disappears when React mounts.
- `GET /api/v1/catalog` returns the Backed instrument list.
- `GET /api/v1/check?crypto_id=36994&quantity=75&destination=USDC_SOLANA`
  returns a sell quote, size quotes, routes and the timeline inputs.
