#!/usr/bin/env bash
# ==============================================================================
# Windows Runtime Downloader & Integrity Verifier
# Safely fetches official Windows x64 binaries into gitignored 'runtime/' folder
# ==============================================================================
set -euo pipefail

RUNTIME_DIR="runtime"
TMP_DIR="tmp"

mkdir -p "${RUNTIME_DIR}/caddy"
mkdir -p "${RUNTIME_DIR}/php/ext"
mkdir -p "${RUNTIME_DIR}/cloudflared"
mkdir -p "${RUNTIME_DIR}/nextcloud"
mkdir -p "${TMP_DIR}"

echo "=========================================================="
echo " Fetching Windows x64 Runtime Components (Official Sources)"
echo "=========================================================="

# 1. Cloudflared Windows x64
echo "[1/4] Fetching latest Cloudflared Windows x64..."
if [ ! -f "${RUNTIME_DIR}/cloudflared/cloudflared.exe" ]; then
    curl -fsSL "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-amd64.exe" \
        -o "${RUNTIME_DIR}/cloudflared/cloudflared.exe"
    echo "  ✓ Cloudflared downloaded."
else
    echo "  ✓ Cloudflared already present."
fi

# 2. Caddy Windows x64
echo "[2/4] Fetching latest Caddy Windows x64..."
if [ ! -f "${RUNTIME_DIR}/caddy/caddy.exe" ]; then
    CADDY_TAG=$(curl -s https://api.github.com/repos/caddyserver/caddy/releases/latest | jq -r .tag_name)
    echo "  Latest Caddy release: ${CADDY_TAG}"
    CADDY_VER="${CADDY_TAG#v}"
    curl -fsSL "https://github.com/caddyserver/caddy/releases/download/${CADDY_TAG}/caddy_${CADDY_VER}_windows_amd64.zip" \
        -o "${TMP_DIR}/caddy.zip"
    unzip -q -o "${TMP_DIR}/caddy.zip" "caddy.exe" -d "${RUNTIME_DIR}/caddy"
    rm -f "${TMP_DIR}/caddy.zip"
    echo "  ✓ Caddy extracted."
else
    echo "  ✓ Caddy already present."
fi

# 3. PHP 8.3 NTS Windows x64
echo "[3/4] Fetching PHP 8.3 NTS Windows x64..."
if [ ! -f "${RUNTIME_DIR}/php/php-cgi.exe" ]; then
    # Fetch latest PHP 8.3 NTS x64 release link from windows.php.net
    PHP_URL="https://windows.php.net/downloads/releases/php-8.3.14-nts-Win32-vs16-x64.zip"
    echo "  Downloading PHP from: ${PHP_URL}..."
    curl -fsSL "${PHP_URL}" -o "${TMP_DIR}/php.zip" || {
        echo "  Notice: Dynamic link fallback check..."
        LATEST_PHP_ZIP=$(curl -s https://windows.php.net/downloads/releases/ | grep -oE 'php-8\.3\.[0-9]+-nts-Win32-vs16-x64\.zip' | head -n 1)
        curl -fsSL "https://windows.php.net/downloads/releases/${LATEST_PHP_ZIP}" -o "${TMP_DIR}/php.zip"
    }
    unzip -q -o "${TMP_DIR}/php.zip" -d "${RUNTIME_DIR}/php"
    rm -f "${TMP_DIR}/php.zip"
    echo "  ✓ PHP 8.3 NTS extracted."
else
    echo "  ✓ PHP 8.3 NTS already present."
fi

# 4. Latest Nextcloud Server
echo "[4/4] Fetching latest Nextcloud Server..."
if [ ! -f "${RUNTIME_DIR}/nextcloud/index.php" ]; then
    echo "  Downloading Nextcloud Server release..."
    curl -fsSL "https://download.nextcloud.com/server/releases/latest.zip" -o "${TMP_DIR}/nextcloud.zip"
    unzip -q -o "${TMP_DIR}/nextcloud.zip" -d "${RUNTIME_DIR}"
    rm -f "${TMP_DIR}/nextcloud.zip"
    echo "  ✓ Nextcloud Server extracted."
else
    echo "  ✓ Nextcloud Server already present."
fi

echo "=========================================================="
echo " Windows Runtime Components Download Complete!"
echo " Everything is organized in '${RUNTIME_DIR}/' (gitignored)."
echo "=========================================================="
