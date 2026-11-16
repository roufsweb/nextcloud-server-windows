---
name: file-organizer
description: >-
  Enforces clean file organization, modular folder boundaries, runtime isolation,
  and artifact hygiene for the Nextcloud Server for Windows project.
---

# Project File Organization Skill

Use this skill when creating, renaming, moving, or refactoring files and folders in this repository.

---

## 1. Directory Structure Blueprint

All files must reside strictly in their designated domain:

```text
nextcloud-server-windows/
├── .agents/                        # Agent configurations, rules, and skills
│   ├── rules/                      # Enforced project rules
│   └── skills/                     # Specialized procedural skills
├── config/                         # Production configuration templates
│   ├── caddy/                      # Caddyfile and web server configs
│   ├── php/                        # php.ini and extension configs
│   └── nextcloud/                  # config.sample.php, autoconfig.php
├── core/                           # Native Rust backend & supervisor
│   ├── Cargo.toml                  # Package manifest
│   └── src/
│       ├── config.rs               # Configuration state & serialization
│       ├── drive.rs                # Windows NTFS ACL permissions & mounts
│       ├── main.rs                 # CLI entry point & IPC bridge
│       ├── supervisor.rs           # Process life cycle manager
│       └── tunnel.rs               # Cloudflare Tunnel integration
├── ui/                             # Standalone Modern Desktop Frontend
│   ├── index.html                  # Main UI entry point
│   ├── css/                        # High-performance modular stylesheets
│   ├── js/                         # State management & IPC controllers
│   └── components/                 # Audited UI components (Dashboard, Logs, Storage)
├── scripts/                        # Automation & pipeline scripts
│   ├── download_windows_runtime.sh # Binary fetcher (official sources only)
│   ├── package_windows.sh          # Distribution zip creator
│   └── audit_frontend.sh          # Frontend size & dead code auditor
├── dist/                           # [Gitignored] Packaged releases
├── runtime/                        # [Gitignored] Windows portable executables
├── data/                           # [Gitignored] Nextcloud user data directory
└── logs/                           # [Gitignored] Server & application logs
```

---

## 2. Organization Rules & Boundaries

1. **No Runtime Blobs in Source Folders**:
   - Executables (`.exe`), DLLs (`.dll`), and zip archives must **never** be placed in `core/`, `ui/`, or `config/`.
   - All external runtime components belong in `runtime/` (which is gitignored).
2. **Frontend Isolation**:
   - `ui/` must remain completely decoupled from operating system APIs; all interactions with Windows drives, services, and tunnels must go through the Rust supervisor IPC / API.
3. **Log & Data Separation**:
   - Application logs must always route to `logs/` and never clutter the root directory or `config/`.
   - Nextcloud data files must strictly reside in `data/` or user-mounted drives.
