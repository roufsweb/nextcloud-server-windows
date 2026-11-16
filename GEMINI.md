# GEMINI.md - Assistant Guidelines

This repository follows the guidelines specified in:
- [AGENTS.md](file:///mnt/data/roufsweb/nextcloud-server-windows/AGENTS.md)
- [.agents/rules/storage_and_system_protection.md](file:///mnt/data/roufsweb/nextcloud-server-windows/.agents/rules/storage_and_system_protection.md)
- [.agents/rules/github_guidelines_and_dmca.md](file:///mnt/data/roufsweb/nextcloud-server-windows/.agents/rules/github_guidelines_and_dmca.md)
- [.agents/rules/frontend_audit_and_debloating.md](file:///mnt/data/roufsweb/nextcloud-server-windows/.agents/rules/frontend_audit_and_debloating.md)
- [.agents/skills/github-ops/SKILL.md](file:///mnt/data/roufsweb/nextcloud-server-windows/.agents/skills/github-ops/SKILL.md)
- [.agents/skills/file-organizer/SKILL.md](file:///mnt/data/roufsweb/nextcloud-server-windows/.agents/skills/file-organizer/SKILL.md)
- [.agents/skills/production-naming/SKILL.md](file:///mnt/data/roufsweb/nextcloud-server-windows/.agents/skills/production-naming/SKILL.md)

All tools must be installed and operated inside `/mnt/data/tools/` using `source /mnt/data/tools/env.sh`. Never write large caches or tools to the root filesystem `/`. Always perform pre-push checks (`.agents/skills/github-ops/scripts/pre_push_check.sh`) and frontend debloat audits (`scripts/audit_frontend.sh`) prior to committing or releasing code. Ensure no secrets, tokens, or copyrighted binaries are committed.
