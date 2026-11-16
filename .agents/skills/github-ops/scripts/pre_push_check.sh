#!/usr/bin/env bash
# ==============================================================================
# GitHub Pre-Push & DMCA Compliance Security Checker
# ==============================================================================
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

ERRORS=0
WARNINGS=0

echo -e "${BLUE}==> [1/4] Checking for Staged Secrets, Tokens, and Private Keys...${NC}"

# Patterns for sensitive tokens, private keys, passwords
SECRET_PATTERNS=(
    "eyJh[A-Za-z0-9_-]{20,}"                     # Cloudflare tunnel token pattern
    "BEGIN (RSA|EC|DSA|OPENSSH|PGP) PRIVATE KEY" # Private keys
    "ghp_[A-Za-z0-9]{36}"                        # GitHub Personal Access Token
    "gho_[A-Za-z0-9]{36}"                        # GitHub OAuth Access Token
    "AIza[0-9A-Za-z\\-_]{35}"                    # Google API Key
    "password[[:space:]]*=[[:space:]]*['\"][^'\"]{6,}['\"]" # Hardcoded passwords
)

# Inspect staged changes and tracked files (ignoring binary files)
for pattern in "${SECRET_PATTERNS[@]}"; do
    MATCHES=$(git grep -EI "$pattern" -- ':!.agents/skills/github-ops/scripts/pre_push_check.sh' ':!tests/' 2>/dev/null || true)
    if [ -n "$MATCHES" ]; then
        echo -e "${RED}[ERROR] Potential secret/token detected matching pattern: $pattern${NC}"
        echo "$MATCHES" | head -n 5
        ERRORS=$((ERRORS + 1))
    fi
done

if [ "$ERRORS" -eq 0 ]; then
    echo -e "${GREEN}  ✓ No hardcoded secrets or tokens detected.${NC}"
fi

echo -e "${BLUE}==> [2/4] Checking for Large Binaries (>10MB) & Git LFS Bloat...${NC}"
# GitHub limits files to 100MB, but best practice is < 10MB to prevent repo bloat
LARGE_FILES=$(git ls-files | while read -r file; do
    if [ -f "$file" ]; then
        size=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file" 2>/dev/null || echo 0)
        if [ "$size" -gt 10485760 ]; then # 10MB
            mb=$((size / 1048576))
            echo "$file (${mb}MB)"
        fi
    fi
done)

if [ -n "$LARGE_FILES" ]; then
    echo -e "${RED}[ERROR] The following files exceed 10MB. Large files should be distributed via GitHub Releases, not git history:${NC}"
    echo "$LARGE_FILES"
    ERRORS=$((ERRORS + 1))
else
    echo -e "${GREEN}  ✓ No oversized binary blobs detected.${NC}"
fi

echo -e "${BLUE}==> [3/4] DMCA & License Compliance Audit...${NC}"
# Check for unapproved binary file extensions in tracked files
FORBIDDEN_EXTENSIONS='(\.exe|\.dll|\.sys|\.msi|\.so|\.dylib|\.iso)$'
TRACKED_BINARIES=$(git ls-files | grep -Ei "$FORBIDDEN_EXTENSIONS" || true)

if [ -n "$TRACKED_BINARIES" ]; then
    echo -e "${RED}[ERROR] Forbidden executable/library binaries tracked in git:${NC}"
    echo "$TRACKED_BINARIES"
    echo -e "${YELLOW}Binaries must be compiled dynamically or fetched during installation via GitHub Releases to avoid DMCA and copyright flags.${NC}"
    ERRORS=$((ERRORS + 1))
else
    echo -e "${GREEN}  ✓ No compiled executable/binary files tracked in repository.${NC}"
fi

# Check for LICENSE file
if [ ! -f "LICENSE" ]; then
    echo -e "${YELLOW}[WARNING] No LICENSE file found in repository root. Open source repositories require a clear license to prevent ambiguity.${NC}"
    WARNINGS=$((WARNINGS + 1))
else
    echo -e "${GREEN}  ✓ LICENSE file is present.${NC}"
fi

echo -e "${BLUE}==> [4/4] Summary...${NC}"
if [ "$ERRORS" -gt 0 ]; then
    echo -e "${RED}FAILED: $ERRORS critical security/DMCA compliance errors found. Aborting push.${NC}"
    exit 1
elif [ "$WARNINGS" -gt 0 ]; then
    echo -e "${YELLOW}PASSED with $WARNINGS warning(s). Proceed with caution.${NC}"
    exit 0
else
    echo -e "${GREEN}PASSED: All GitHub guidelines, security, and DMCA compliance checks passed!${NC}"
    exit 0
fi
