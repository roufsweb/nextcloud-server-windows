#!/usr/bin/env bash
# ==============================================================================
# Package Windows All-In-One Distribution
# Prepares portable zip bundle ready for Windows users / GitHub Releases
# ==============================================================================
set -euo pipefail

DIST_DIR="dist/nextcloud-windows-aio"
RELEASE_ZIP="dist/nextcloud-windows-aio-v1.0.0-portable-x64.zip"

echo "=========================================================="
echo " Packaging Nextcloud Server for Windows (All-In-One)"
echo "=========================================================="

rm -rf "${DIST_DIR}" "${RELEASE_ZIP}"
mkdir -p "${DIST_DIR}"
mkdir -p "${DIST_DIR}/data"
mkdir -p "${DIST_DIR}/logs"
mkdir -p "${DIST_DIR}/tmp"
mkdir -p "dist"

# Copy configurations
echo "-> Copying configuration templates..."
cp -r config "${DIST_DIR}/"

# Copy UI components
echo "-> Copying standalone desktop UI..."
cp -r ui "${DIST_DIR}/"

# Copy launcher and docs
echo "-> Copying documentation and licenses..."
cp README.md "${DIST_DIR}/"
cp LICENSE "${DIST_DIR}/"
cp THIRD_PARTY_LICENSES.md "${DIST_DIR}/"

# Copy compiled Rust executable
if [ -f "core/target/x86_64-pc-windows-gnu/release/nextcloud-aio.exe" ]; then
    echo "-> Copying cross-compiled Windows binary..."
    cp "core/target/x86_64-pc-windows-gnu/release/nextcloud-aio.exe" "${DIST_DIR}/nextcloud-aio.exe"
elif [ -f "core/target/release/nextcloud-aio.exe" ]; then
    cp "core/target/release/nextcloud-aio.exe" "${DIST_DIR}/nextcloud-aio.exe"
elif [ -f "core/target/release/nextcloud-aio" ]; then
    echo "-> Including native binary for local testing..."
    cp "core/target/release/nextcloud-aio" "${DIST_DIR}/nextcloud-aio"
fi

# Copy runtime binaries if present
if [ -d "runtime" ]; then
    echo "-> Copying runtime components..."
    cp -r runtime/* "${DIST_DIR}/" 2>/dev/null || true
fi

# Add quick start Windows batch script
cat << 'EOF' > "${DIST_DIR}/start-server.bat"
@echo off
title Nextcloud Server for Windows (All-In-One)
echo ==========================================================
echo Starting Nextcloud All-In-One Server & Desktop Dashboard...
echo ==========================================================
if exist "nextcloud-aio.exe" (
    start http://localhost:8081
    nextcloud-aio.exe start
) else (
    echo [ERROR] nextcloud-aio.exe not found in current directory.
    pause
)
EOF

cat << 'EOF' > "${DIST_DIR}/open-dashboard.bat"
@echo off
echo Opening Nextcloud AIO Desktop Dashboard...
start http://localhost:8081
EOF

echo "✓ Distribution folder prepared at: ${DIST_DIR}"
echo "To package final zip archive, run: (cd dist && zip -r nextcloud-windows-aio-v1.0.0-portable-x64.zip nextcloud-windows-aio)"
