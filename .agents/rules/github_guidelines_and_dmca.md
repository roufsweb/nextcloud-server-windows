# GitHub Guidelines, Security, & DMCA Anti-Violation Rules

This document outlines mandatory operational constraints for all agents and contributors working on the **Nextcloud Server for Windows (All-In-One)** repository. These rules strictly enforce compliance with GitHub's Terms of Service, Acceptable Use Policies, DMCA/Copyright regulations, and security standards to prevent account suspension, repository strikes, or legal takedowns.

---

## 1. Zero-Secrets Policy (GitHub Secret Scanning Compliance)

GitHub automatically scans commits, pull requests, and public issues for credentials. Committing any active or dormant credentials triggers immediate security alerts, revoked tokens, or account flags.

### Strict Rules:
- **Cloudflare Tunnel Tokens**: NEVER commit raw `cloudflared` tunnel tokens, JWT credentials, or origin certs (`cert.pem`). Always use environment variables (`CLOUDFLARE_TUNNEL_TOKEN`) or a gitignored `secrets/` directory with `.token` extension.
- **Database Passwords**: NEVER hardcode default or production passwords for MariaDB/PostgreSQL/Redis in configuration files committed to git. Use template files like `config.sample.php` or `env.example`.
- **Private Keys & Certificates**: No SSL/TLS private keys (`.key`, `.pem`, `.pfx`) may ever enter the git tree.
- **Pre-Commit Enforcement**: Always execute `scripts/pre_push_check.sh` before staging or pushing code.

---

## 2. DMCA & Copyright Violation Prevention Pipeline

GitHub strictly enforces DMCA 17 U.S.C. § 512(c). Three strikes results in permanent account and organization bans.

### Prohibited Content:
1. **Proprietary Commercial Software**: Never commit proprietary Windows DLLs, proprietary drivers, or closed-source commercial libraries.
2. **Third-Party Binary Blobs**: Never commit precompiled binary executables (`.exe`, `.dll`, `.sys`, `.msi`) into git. Binary dependencies must be downloaded dynamically during build/runtime from official upstream release URLs with SHA-256 integrity verification.
3. **Trademark Violations**: 
   - Nextcloud is a registered trademark of Nextcloud GmbH.
   - This project must clearly identify itself as an unofficial, community-driven pack.
   - Always include the Nextcloud trademark disclaimer in user-facing documentation.
4. **License Incompatibilities**:
   - Nextcloud Server is licensed under **GNU AGPLv3**.
   - Any bundled PHP code, server hooks, or apps interacting directly with Nextcloud must be license-compatible (AGPLv3 or compatible open source licenses).
   - Rust launcher / wrapper code can be licensed under MIT or Apache 2.0 as long as it communicates with Nextcloud as an independent external process supervisor via standard network/IPC protocols.
5. **No Bypasses or DRM Circumvention**:
   - Under no circumstances shall this repository include code, tools, or instructions designed to circumvent digital rights management (DMCA § 1201), paywalls, or unauthorized access controls.

---

## 3. GitHub Acceptable Use & Storage Restrictions

1. **Repository Size & File Limits**:
   - GitHub has a strict 100 MB per-file limit and recommends keeping repositories under 1 GB total.
   - Never use GitHub git tracking as a CDN or binary asset storage for Nextcloud releases (which exceed 200MB+).
   - All server packs, pre-bundled PHP archives, and Windows runtime installers must be published via **GitHub Releases** or external object storage, NOT in git commit history.
2. **Bandwidth & Actions Abuse**:
   - GitHub Actions must not be used for cryptomining, hosting proxies, tunneling raw traffic, or excessive bandwidth consumption.
   - CI/CD workflows must only be used for automated building, testing, linting, and compiling release binaries.

---

## 4. Pre-Push Verification Checklist

Before pushing any commit to a remote GitHub branch, the agent or developer must verify:
- [ ] No `.env`, `.token`, `.pem`, or credentials exist in `git status`.
- [ ] No file exceeds 10 MB in the staging index.
- [ ] License headers and upstream attributions are in place.
- [ ] `scripts/pre_push_check.sh` passes with 0 errors.
