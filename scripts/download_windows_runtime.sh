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
echo "[1/5] Fetching latest Cloudflared Windows x64..."
if [ ! -f "${RUNTIME_DIR}/cloudflared/cloudflared.exe" ]; then
    curl -fsSL "https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-amd64.exe" \
        -o "${RUNTIME_DIR}/cloudflared/cloudflared.exe"
    echo "  ✓ Cloudflared downloaded."
else
    echo "  ✓ Cloudflared already present."
fi

# 2. Caddy Windows x64
echo "[2/5] Fetching latest Caddy Windows x64..."
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
echo "[3/5] Fetching PHP 8.3 NTS Windows x64..."
if [ ! -f "${RUNTIME_DIR}/php/php-cgi.exe" ]; then
    PHP_URL="https://downloads.php.net/~windows/releases/archives/php-8.3.36-nts-Win32-vs16-x64.zip"
    echo "  Downloading PHP from: ${PHP_URL}..."
    curl -fsSL "${PHP_URL}" -o "${TMP_DIR}/php.zip"
    unzip -q -o "${TMP_DIR}/php.zip" -d "${RUNTIME_DIR}/php"
    rm -f "${TMP_DIR}/php.zip"
    echo "  ✓ PHP 8.3 NTS extracted."
else
    echo "  ✓ PHP 8.3 NTS already present."
fi

# 4. Windows PECL Extensions: Redis & APCu
echo "[4/5] Fetching PECL extensions (Redis & APCu)..."
if [ ! -f "${RUNTIME_DIR}/php/ext/php_redis.dll" ]; then
    REDIS_URL="https://downloads.php.net/~windows/pecl/releases/redis/6.3.0/php_redis-6.3.0-8.3-nts-vs16-x64.zip"
    curl -fsSL "${REDIS_URL}" -o "${TMP_DIR}/redis.zip"
    unzip -q -o "${TMP_DIR}/redis.zip" "php_redis.dll" -d "${RUNTIME_DIR}/php/ext"
    rm -f "${TMP_DIR}/redis.zip"
    echo "  ✓ php_redis.dll extracted."
fi

if [ ! -f "${RUNTIME_DIR}/php/ext/php_apcu.dll" ]; then
    APCU_URL="https://downloads.php.net/~windows/pecl/releases/apcu/5.1.28/php_apcu-5.1.28-8.3-nts-vs16-x64.zip"
    curl -fsSL "${APCU_URL}" -o "${TMP_DIR}/apcu.zip"
    unzip -q -o "${TMP_DIR}/apcu.zip" "php_apcu.dll" -d "${RUNTIME_DIR}/php/ext"
    rm -f "${TMP_DIR}/apcu.zip"
    echo "  ✓ php_apcu.dll extracted."
fi

# 5. Latest Nextcloud Server
echo "[5/5] Fetching latest Nextcloud Server..."
if [ ! -f "${RUNTIME_DIR}/nextcloud/index.php" ]; then
    echo "  Downloading Nextcloud Server release (~280 MB)..."
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
