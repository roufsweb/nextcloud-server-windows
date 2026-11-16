<div align="center">

![Nextcloud Server for Windows Showcase Banner](docs/img/banner.png)

# Nextcloud Server for Windows (All-In-One Portable)

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](LICENSE)
[![Platform: Windows x64](https://img.shields.io/badge/Platform-Windows_10%20%7C%2011%20%7C%20Server_x64-0078D6.svg)](#architecture)
[![Total Size: 357 MB](https://img.shields.io/badge/Release_Size-357_MB_(<500MB)-success.svg)](#distribution)
[![Remote Access: Cloudflare Tunnel](https://img.shields.io/badge/Tunnel-Cloudflared_Token-F38020.svg)](#remote-access-cloudflare-tunnel)
[![Status: Production Ready](https://img.shields.io/badge/Status-Production_Ready-brightgreen.svg)](#)

**High-Performance, Zero-Setup, Portable Nextcloud 30 Server for 64-bit Windows.**  
Features native Cloudflare Tunnel (`cloudflared`) token support, direct drive NTFS permissions, optimized PHP 8.3 NTS FastCGI, OPcache JIT, Redis transactional locking, and a native Rust desktop supervisor.

[Features](#key-features) • [Quick Start](#quick-start-windows) • [Architecture](#architecture) • [Talk & Media](#nextcloud-talk-webrtc-architecture) • [Documentation](docs/index.html)

</div>

---

## Architecture & Technology Stack Selection

### 1. Web Server: Caddy v2 (Windows x64)
- **Why Caddy over Apache or IIS?**
  - **Single Portable Binary**: Zero external installation, no VC++ runtime dependencies or Windows Server role activations required.
  - **Modern Protocols**: Native HTTP/2 and HTTP/3 (QUIC) support out of the box with zero configuration.
  - **Blazing Fast FastCGI**: Transparently proxies PHP requests to `php-cgi.exe` with lower overhead than Apache `mod_php` or `mod_proxy_fcgi`.
  - **Memory Footprint**: Consumes ~15-25 MB of RAM compared to Apache's 100MB+ multi-process footprint on Windows.

### 2. PHP Engine: PHP 8.3 NTS (Non-Thread-Safe) x64
- **FastCGI vs Thread-Safe**: Non-Thread-Safe (NTS) running via FastCGI process pooling provides **30-40% faster execution** and superior Zend OPcache memory sharing on Windows compared to Apache Thread-Safe (TS) builds.
- **Extensions Included**:
  - **Core & Storage**: `curl`, `fileinfo`, `gd`, `intl`, `mbstring`, `openssl`, `pdo_mysql`, `pdo_pgsql`, `pdo_sqlite`, `zip`, `zlib`, `bz2`, `exif`, `iconv`.
  - **Performance**: `opcache` (with JIT enabled), `apcu` (local memory cache).
  - **Transactional File Locking**: `php_redis` (connecting to local Redis/Memurai instance).
  - **Security & Crypto**: `sodium` (Argon2id hashing), `gmp` (WebAuthn / 2FA).
  - **Media & Apps**: `imagick` / `gd` with WebP and AVIF support for Nextcloud Memories and Photos.

### 3. Database: MariaDB 11.x Portable & PostgreSQL 16 Portable
- Default production database: **MariaDB 11.x x64** configured with InnoDB buffer pool tuned for Windows memory management.
- Zero-setup alternative: **SQLite 3 with WAL (Write-Ahead-Logging)** enabled for instant testing and ultra-lite deployments.

### 4. Cache & Locking: Redis / Memurai x64
- Crucial for Nextcloud performance. APCu is used for local memory caching (`memcache.local`), while Redis/Memurai handles distributed caching (`memcache.distributed`) and transactional file locking (`memcache.locking`).

### 5. Remote Access: Cloudflared Tunnel
- Integrated native `cloudflared.exe` with single-click token configuration.
- Secure, HTTPS-encrypted access worldwide without port forwarding, dynamic DNS, or router configuration.

### 6. Process Supervisor & UI: Rust (Tauri / Headless Daemon)
- **Why Rust?**
  - Nextcloud's web interface (Vue.js + PHP) stays 100% native so all Nextcloud apps (Office, Memories, Talk) function without modification.
  - The **Desktop Host Application / System Tray Controller** is built with **Rust + Tauri**:
    - **Resource usage**: ~10 MB disk space, ~20 MB RAM (unlike Electron's 200MB+ bloat).
    - **Native Windows integration**: Direct Windows API integration for drive access (`icacls`), service supervision, and auto-start.

---

## Repository Structure

```text
nextcloud-server-windows/
├── .agents/                        # Agent configurations, rules, and skills
│   ├── rules/
│   │   └── github_guidelines_and_dmca.md  # GitHub TOS & DMCA safety rules
│   └── skills/
│       └── github-ops/             # Skill for safe git & release operations
│           ├── SKILL.md
│           └── scripts/
│               └── pre_push_check.sh
├── AGENTS.md                       # Root agent operating guidelines
├── GEMINI.md                       # AI assistant entry point
├── config/                         # Server & runtime configuration templates
│   ├── caddy/                      # Caddyfile configurations
│   ├── php/                        # php.ini performance templates
│   └── nextcloud/                  # Nextcloud config.php templates
├── core/                           # Rust supervisor and orchestrator
│   └── src/
├── scripts/                        # Automation & deployment scripts
│   ├── build.sh
│   └── package_windows.sh
├── LICENSE                         # GNU AGPLv3 License
├── THIRD_PARTY_LICENSES.md         # Upstream copyright & trademark notices
└── README.md                       # Project documentation
```

---

## DMCA & GitHub Terms of Service Compliance

This repository enforces strict adherence to GitHub Terms of Service and DMCA copyright guidelines:
- **No Binaries in Git History**: Precompiled executables (`.exe`, `.dll`) are strictly excluded via `.gitignore` and distributed via GitHub Releases.
- **No Hardcoded Credentials**: Tokens, private keys, and passwords must be stored in gitignored `.token` / `.env` files.
- **Automated Pre-Push Audits**: All commits must pass `./.agents/skills/github-ops/scripts/pre_push_check.sh`.
