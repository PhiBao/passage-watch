#!/usr/bin/env bash
set -euo pipefail

# In containers the key arrives as an environment variable (Fly secret), and
# there is no .env file. Locally, `dotenvy::dotenv_override()` makes a .env file
# authoritative when one exists. So do NOT strip the environment here.
if [ -f /app/.env ]; then
  echo "using .env for local configuration"
else
  echo "using environment for configuration (no .env present)"
fi

exec passage-watch
