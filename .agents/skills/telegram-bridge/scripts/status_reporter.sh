#!/usr/bin/env bash
# ==============================================================================
# Telegram Progress & Status Reporter Helper
# Sends formatted status cards or in-place edits to prevent chat clutter
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLI="${SCRIPT_DIR}/telegram_cli.php"

MESSAGE="${1:-"Agent is working..."}"

php "${CLI}" progress-update "${MESSAGE}"
