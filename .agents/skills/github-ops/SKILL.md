---
name: github-ops
description: >-
  Enforces safe GitHub operations including upload, download, push, release asset
  distribution, and the DMCA anti-violation compliance pipeline.
---

# GitHub Operations & DMCA Compliance Skill

Use this skill whenever interacting with GitHub repositories, pushing code, pulling changes, releasing binaries, or verifying copyright and security compliance to avoid account suspensions and DMCA takedowns.

---

## 1. Safety Protocols & Anti-Ban Rules

GitHub enforces automated scanning and aggressive automated enforcement for:
1. **Secret Leakage**: Tokens, credentials, tunnel credentials.
2. **DMCA / Copyright Strikes**: Checking in proprietary or non-compliant binary assets.
3. **Bandwidth / Quota Abuse**: Checking in massive archives (>100MB files, huge git trees).

Before every push or commit, execute the validation script:
```bash
./.agents/skills/github-ops/scripts/pre_push_check.sh
```

---

## 2. Safe Push Workflow

### Step 1: Pre-Commit Hygiene
Ensure that no sensitive or binary files are staged:
```bash
git status --ignored
```
Verify that `.gitignore` catches all `.token`, `.env`, `.key`, `data/`, and compiled binaries (`*.exe`, `*.dll`).

### Step 2: Run Compliance Audit
```bash
./.agents/skills/github-ops/scripts/pre_push_check.sh
```
If errors occur (e.g. secret pattern detected or large file tracked), resolve them immediately before proceeding.

### Step 3: Staging and Atomic Commits
Commit with clear, descriptive messages:
```bash
git add .
git commit -m "feat: <description of change>"
```

### Step 4: Pushing to Remote
Push using standard authenticated credentials or GitHub CLI:
```bash
git push origin <branch-name>
```

---

## 3. Safe Download & Upstream Asset Handling

When downloading external dependencies for the Windows Nextcloud Server (such as Nextcloud server archive, PHP Windows binaries, Caddy server, or Cloudflared):
1. **Never commit downloaded archives to git.** All downloads must target directories listed in `.gitignore` (such as `tmp/`, `runtime/`, or `vendor/bin/`).
2. **Verify SHA-256 Checksums**: Always check official checksums / PGP signatures from official sources (e.g., `download.nextcloud.com`, `windows.php.net`, `caddyserver.com`, `github.com/cloudflare/cloudflared/releases`).
3. Example automated fetch:
   ```bash
   # Download Nextcloud to untracked directory
   curl -fsSL https://download.nextcloud.com/server/releases/latest.zip -o tmp/nextcloud-latest.zip
   # Verify SHA256 against official Nextcloud checksum
   ```

---

## 4. Release Asset Upload Pipeline (GitHub Releases)

Instead of bloating git history with large Windows installers or portable zip bundles:
1. Use GitHub CLI (`gh release`) to distribute compiled application bundles.
2. Example release command:
   ```bash
   gh release create v1.0.0 ./dist/nextcloud-windows-aio-v1.0.0.zip \
     --title "Nextcloud Server for Windows v1.0.0" \
     --notes "Automated all-in-one portable release for Windows x64."
   ```

---

## 5. DMCA Violation Prevention Checklist

- **AGPLv3 Compliance**: Nextcloud is licensed under AGPLv3. Any modifications to Nextcloud core or bundled apps must make source code available under AGPLv3.
- **Rust Wrapper Independence**: The Rust supervisor / GUI controller acts as an independent external orchestrator (communicating via CLI/IPC/HTTP), permitting MIT/Apache 2.0 licensing.
- **Attribution & Third-Party Notices**: Maintain an up-to-date `THIRD_PARTY_LICENSES.md` documenting upstream projects: Nextcloud, Caddy, PHP, Cloudflared, and MariaDB.
- **Trademark Notice**: Always state: *"Nextcloud is a registered trademark of Nextcloud GmbH. This project is an independent community project and is not affiliated with or endorsed by Nextcloud GmbH."*
