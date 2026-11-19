#!/usr/bin/env bash
set -euo pipefail
# The key must come from the .env file. The shell may carry a stale CMC_API_KEY;
# dotenvy_override() makes the file authoritative, so strip the shell var first.
unset CMC_API_KEY || true

exec passage-watch
