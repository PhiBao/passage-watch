#!/usr/bin/env bash
# Reproduce the live calls used by Passage Watch with your own campaign key.
# Usage: cp .env.example .env && edit .env && bash docs/evidence/reproduce.sh
set -euo pipefail

KEY="${CMC_API_KEY:-}"
if [ -z "$KEY" ] && [ -f .env ]; then
  KEY="$(grep -E '^CMC_API_KEY=' .env | head -1 | cut -d= -f2-)"
fi
if [ -z "$KEY" ] || [ "$KEY" = "replace_with_your_key" ]; then
  echo "Set CMC_API_KEY in .env first." >&2
  exit 1
fi

call() {
  local path="$1"; shift
  echo "### GET $path $*"
  curl -sS --fail-with-body \
    -H "X-CMC_PRO_API_KEY: $KEY" \
    -H "Accept: application/json" \
    --get --data-urlencode "$@" \
    "https://pro-api.coinmarketcap.com$path"
  echo
}

call /v1/key/info x=x
call /v5/real-world-assets/map symbol=AAPL limit=5
call /v5/real-world-assets/quotes/latest rwa_id=3
call /v2/cryptocurrency/info id=36994
call /v3/cryptocurrency/quotes/historical id=36994 interval=1d count=3

echo "### SOLANA parsed mint (read-only)"
curl -sS --fail-with-body https://api.mainnet-beta.solana.com \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"getAccountInfo","params":["XsbEhLAtcf6HdfpFZ5xEMdqW8nfAvcsP5bdudRLJzJp",{"encoding":"jsonParsed"}]}' \
  | python3 -c 'import sys,json; d=json.load(sys.stdin); exts=d["result"]["value"]["data"]["parsed"]["info"]["extensions"]; print([e for e in exts if e["extension"]=="scaledUiAmountConfig"])'

echo "### JUPITER read-only quote (no taker)"
curl -sS --fail-with-body 'https://api.jup.ag/swap/v2/order?inputMint=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v&outputMint=XsbEhLAtcf6HdfpFZ5xEMdqW8nfAvcsP5bdudRLJzJp&amount=100000000&slippageBps=50'
echo
