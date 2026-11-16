#!/usr/bin/env bash
# ==============================================================================
# Telegram Bridge Listener Daemon
# Loops long-polling without LLM token usage until an authorized message arrives
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLI="${SCRIPT_DIR}/telegram_cli.php"

while true; do
    RESULT=$(php "${CLI}" wait-message 45)
    TYPE=$(echo "${RESULT}" | jq -r .type 2>/dev/null || echo "")

    if [ "${TYPE}" = "message" ] || [ "${TYPE}" = "callback" ]; then
        echo "${RESULT}"
        exit 0
    fi
    sleep 1
done
