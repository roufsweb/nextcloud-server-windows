# AGENTS.md - Agent Operating Guidelines

Welcome to the **Nextcloud Server for Windows (All-In-One)** repository.

All AI agents working on this project must strictly comply with the following core rules:

## 1. Compliance & Security Rules
- **Follow GitHub Guidelines & Policies**: Adhere to the rules in [github_guidelines_and_dmca.md](file:///.agents/rules/github_guidelines_and_dmca.md).
- **Never commit secrets or tokens**: Cloudflare tunnel tokens, passwords, private keys, API credentials must never be committed.
- **DMCA / Copyright Compliance**: Only use open source, license-compatible code and dependencies. Never commit proprietary binaries or copyrighted blobs. Verify upstream licenses before importing dependencies.
- **Repository Size Management**: Never commit heavy compiled archives or executable files into git history. Direct them to GitHub Releases.

## 2. Project Architecture Principles
- **Target Platform**: Windows 10 / 11 / Server 2019+ (64-bit), developed and cross-compiled initially on Linux.
- **Core Runtime**: Ultra-lite portable architecture using native Windows binaries:
  - **Reverse Proxy / Web Server**: Caddy (Native x64 binary) with FastCGI.
  - **PHP Engine**: PHP 8.3 NTS (Non-Thread-Safe) x64 with OPcache + APCu.
  - **Database**: Portable MariaDB / PostgreSQL x64 or SQLite3 (with WAL mode) for minimal footprint.
  - **Cache & Transactional Locking**: Redis / Memurai x64 for high performance sync.
  - **Tunnel**: Cloudflared x64 for secure token-based external access without port forwarding.
  - **Process Supervisor & UI**: Native Rust application (Tauri / lightweight daemon) for system tray, service orchestration, direct drive permission handling (Windows ACLs), and auto-updates.
- **High Performance Nextcloud Config**: Maximize OPcache, APCu memory caching, Redis transactional file locking, large upload chunking, and memory limits.

## 3. Workflow Procedures
- Always run the GitHub Ops check script before pushing code:
  ```bash
  .agents/skills/github-ops/scripts/pre_push_check.sh
  ```
- Use the `github-ops` skill for release packaging, repository push, pull, and integrity validation.
