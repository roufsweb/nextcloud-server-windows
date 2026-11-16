#!/usr/bin/env bash
# ==============================================================================
# Package Windows All-In-One Distribution
# Prepares portable zip bundle ready for Windows users / GitHub Releases
# ==============================================================================
set -euo pipefail

DIST_DIR="dist/nextcloud-windows-aio"
RELEASE_ZIP="dist/nextcloud-windows-aio-portable.zip"

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

# Copy launcher and docs
echo "-> Copying documentation and licenses..."
cp README.md "${DIST_DIR}/"
cp LICENSE "${DIST_DIR}/"
cp THIRD_PARTY_LICENSES.md "${DIST_DIR}/"

# Copy runtime binaries if present
if [ -d "runtime" ]; then
    echo "-> Copying runtime components..."
    cp -r runtime/* "${DIST_DIR}/" 2>/dev/null || true
fi

# Add quick start Windows batch script
cat << 'EOF' > "${DIST_DIR}/start-server.bat"
@echo off
title Nextcloud Server for Windows (All-In-One)
echo Starting Nextcloud All-In-One Server...
if exist "nextcloud-aio.exe" (
    nextcloud-aio.exe start
) else (
    echo [ERROR] nextcloud-aio.exe not found in current directory.
    pause
)
EOF

echo "✓ Distribution folder prepared at: ${DIST_DIR}"
echo "To create final release zip, run: cd dist && zip -r nextcloud-windows-aio-portable.zip nextcloud-windows-aio"
