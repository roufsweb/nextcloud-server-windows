#!/usr/bin/env bash
# ==============================================================================
# Build Windows Production Installer (Setup.exe) using NSIS
# ==============================================================================
set -eu
set -o pipefail 2>/dev/null || true

BASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${BASE_DIR}"

source /mnt/data/tools/env.sh 2>/dev/null || true

echo "=========================================================="
echo " Building Nextcloud Server for Windows Production Installer"
echo "=========================================================="

# 1. Verify NSIS is available
if ! command -v makensis &> /dev/null; then
    echo "ERROR: makensis (NSIS) not found in PATH."
    exit 1
fi

# 2. Re-stage portable distribution in dist/nextcloud-windows-aio
echo "-> Staging portable distribution files..."
./scripts/package_windows.sh

# 3. Ensure logo.ico is present
if [ ! -f "docs/img/logo.ico" ] && [ -f "docs/img/logo.png" ]; then
    echo "-> Generating docs/img/logo.ico from logo.png..."
    ffmpeg -y -i docs/img/logo.png -vf scale=256:256 docs/img/logo.ico
fi

# 4. Compile NSIS installer
echo "-> Compressing and compiling Windows Setup executable (NSIS)..."
makensis installer/installer.nsi

SETUP_EXE="dist/nextcloud-windows-aio-v1.0.0-setup-x64.exe"
if [ -f "${SETUP_EXE}" ]; then
    SIZE_MB=$(du -m "${SETUP_EXE}" | cut -f1)
    echo "✓ Production installer generated successfully: ${SETUP_EXE} (${SIZE_MB} MB)"
    (cd dist && sha256sum "nextcloud-windows-aio-v1.0.0-setup-x64.exe" >> SHA256SUMS.txt)
else
    echo "ERROR: Setup executable was not generated."
    exit 1
fi
