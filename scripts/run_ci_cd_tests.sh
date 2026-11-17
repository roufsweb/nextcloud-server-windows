#!/usr/bin/env bash
# ==============================================================================
# Nextcloud Server for Windows (All-In-One) - CI/CD Integration Test Suite
# Runs extraction verification, supervisor API test, multi-drive stress tests,
# Nextcloud config audits, and high-bitrate media benchmarks.
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_DIR}"

if [ -f "/mnt/data/tools/env.sh" ]; then
    # shellcheck disable=SC1091
    source /mnt/data/tools/env.sh
fi

echo "=========================================================="
echo " Running Automated CI/CD & Media Benchmark Suite"
echo "=========================================================="

python3 tests/ci_cd_test_runner.py
