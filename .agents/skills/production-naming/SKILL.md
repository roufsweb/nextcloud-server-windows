---
name: production-naming
description: >-
  Standardizes production-grade naming conventions across Rust code, frontend
  components, scripts, configuration files, and release artifacts.
---

# Production Grade Naming Standards

This skill documents and enforces standard naming conventions across the entire project codebase.

---

## 1. Rust Backend Standards

| Entity | Convention | Example |
| :--- | :--- | :--- |
| **Crate Name** | `kebab-case` | `nextcloud-windows-aio-core` |
| **Binary Target** | `kebab-case` | `nextcloud-aio` (`nextcloud-aio.exe`) |
| **Source Files & Modules** | `snake_case` | `drive_manager.rs`, `supervisor.rs` |
| **Structs, Enums, Traits** | `PascalCase` | `ServerSupervisor`, `AppConfig`, `TunnelState` |
| **Functions & Methods** | `snake_case` | `grant_drive_permissions()`, `start_service()` |
| **Variables & Fields** | `snake_case` | `server_port`, `external_drives` |
| **Constants & Statics** | `SCREAMING_SNAKE_CASE` | `CONFIG_FILE_NAME`, `MAX_LOG_BUFFER_SIZE` |

---

## 2. Frontend / UI Standards

| Entity | Convention | Example |
| :--- | :--- | :--- |
| **Component Files** | `kebab-case.js` / `.html` | `storage-gauge.js`, `log-viewer.js` |
| **Stylesheets** | `kebab-case.css` | `dashboard-layout.css`, `theme-dark.css` |
| **CSS Classes** | `kebab-case` (BEM) | `.storage-card`, `.log-console__line--error` |
| **JavaScript Variables & Functions** | `camelCase` | `refreshDriveList()`, `activeTab` |
| **Event Names** | `kebab-case` | `drive-mounted`, `tunnel-status-changed` |

---

## 3. Scripts & CLI Commands

| Entity | Convention | Example |
| :--- | :--- | :--- |
| **Shell Scripts** | `snake_case.sh` | `download_windows_runtime.sh`, `package_windows.sh` |
| **Windows Batch Scripts** | `snake_case.bat` | `start_server.bat` |
| **CLI Subcommands** | `kebab-case` | `start`, `status`, `set-token`, `grant-drive` |

---

## 4. Release & Packaging Artifacts

All published distribution artifacts must strictly adhere to the versioned naming schema:

* **Portable Bundle**: `nextcloud-windows-aio-v<VERSION>-portable-x64.zip`
  * *Example*: `nextcloud-windows-aio-v1.0.0-portable-x64.zip`
* **Standalone Binary**: `nextcloud-aio.exe`
* **Checksum Manifest**: `SHA256SUMS.txt`
