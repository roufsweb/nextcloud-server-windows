# Storage, System Protection, & Tool Location Rule

This rule is strictly mandatory to protect the host Ubuntu OS drive, aaPanel runtime, and limited root partition storage.

---

## 1. Zero Root Drive Pollution Policy

- **Target Storage Disk**: `/mnt/data` (High-capacity SSD).
- **Prohibited Root Modification**: Never download tools, dependencies, compilation toolchains, package caches, or large datasets to `/` (e.g., `/root`, `/tmp`, `/var`, `/home` on `/dev/sda2`).
- **All Downloaded Tools**: Must strictly reside in `/mnt/data/tools/`.
  - Toolchain binaries: `/mnt/data/tools/cargo`, `/mnt/data/tools/rustup`, `/mnt/data/tools/pnpm`, `/mnt/data/tools/uv`.
  - Package caches: `/mnt/data/.cache/`.
- **Environment Configuration**: Always source `/mnt/data/tools/env.sh` before executing development commands.

---

## 2. System File Protection

- **Do Not Touch System Services**: Do not alter nginx, apache, mysql, or php services managed by aaPanel or the host OS.
- **Standalone Development**: All servers (Caddy, PHP-CGI, Cloudflared) must run inside project-isolated runtimes or portable testing directories under `/mnt/data/`.
