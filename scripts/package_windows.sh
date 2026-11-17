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

# Copy compiled Rust executables
if [ -f "core/target/x86_64-pc-windows-gnu/release/nextcloud-aio.exe" ]; then
    echo "-> Copying cross-compiled Windows supervisor binary..."
    cp "core/target/x86_64-pc-windows-gnu/release/nextcloud-aio.exe" "${DIST_DIR}/nextcloud-aio.exe"
fi

if [ -f "core/target/x86_64-pc-windows-gnu/release/nextcloud-aio-gui.exe" ]; then
    echo "-> Copying cross-compiled Windows native GUI subsystem binary..."
    cp "core/target/x86_64-pc-windows-gnu/release/nextcloud-aio-gui.exe" "${DIST_DIR}/nextcloud-aio-gui.exe"
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
echo Starting Nextcloud Server & Standalone Desktop GUI Window...
echo ==========================================================
if exist "nextcloud-aio-gui.exe" (
    start "" "nextcloud-aio-gui.exe"
) else if exist "nextcloud-aio.exe" (
    start "" "nextcloud-aio.exe" start --gui
) else (
    echo [ERROR] Nextcloud AIO binary not found in current directory.
    pause
)
EOF

cat << 'EOF' > "${DIST_DIR}/open-dashboard.bat"
@echo off
echo Opening Nextcloud AIO Standalone Desktop GUI Window...
if exist "nextcloud-aio.exe" (
    nextcloud-aio.exe gui
) else if exist "nextcloud-aio-gui.exe" (
    start "" "nextcloud-aio-gui.exe"
)
EOF

cat << 'EOF' > "${DIST_DIR}/Nextcloud-AIO.vbs"
Set WshShell = CreateObject("WScript.Shell")
WshShell.Run "nextcloud-aio-gui.exe", 0, False
EOF

echo "-> Compressing portable distribution package..."
(cd dist && rm -f "nextcloud-windows-aio-v1.0.0-portable-x64.zip" && zip -9 -r -q "nextcloud-windows-aio-v1.0.0-portable-x64.zip" nextcloud-windows-aio)
(cd dist && sha256sum "nextcloud-windows-aio-v1.0.0-portable-x64.zip" > SHA256SUMS.txt)

echo "✓ Distribution archive packaged at: ${RELEASE_ZIP}"
cat dist/SHA256SUMS.txt
