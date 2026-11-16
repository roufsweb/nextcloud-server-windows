#!/usr/bin/env bash
# ==============================================================================
# Frontend Audit & Debloating Verification Script
# Validates asset size budgets, dead components, and zero-bloat compliance
# ==============================================================================
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

UI_DIR="ui"
MAX_UI_SIZE_KB=2500 # 2.5 MB maximum allowed for compiled UI
ERRORS=0

echo -e "${BLUE}==> [1/3] Checking Frontend Asset Size Budget (<${MAX_UI_SIZE_KB} KB)...${NC}"

if [ -d "${UI_DIR}" ]; then
    UI_SIZE_KB=$(du -sk "${UI_DIR}" | awk '{print $1}')
    if [ "${UI_SIZE_KB}" -gt "${MAX_UI_SIZE_KB}" ]; then
        echo -e "${RED}[ERROR] UI directory size (${UI_SIZE_KB} KB) exceeds maximum budget (${MAX_UI_SIZE_KB} KB)!${NC}"
        ERRORS=$((ERRORS + 1))
    else
        echo -e "${GREEN}  ✓ UI size (${UI_SIZE_KB} KB) is well within budget (<${MAX_UI_SIZE_KB} KB).${NC}"
    fi
else
    echo -e "${YELLOW}  [!] UI directory '${UI_DIR}' not yet populated.${NC}"
fi

echo -e "${BLUE}==> [2/3] Auditing for Monolithic Unused Packages / Heavy Bloat...${NC}"
FORBIDDEN_PATTERNS=(
    "node_modules"
    "font-awesome"
    "material-design-icons"
    "bootstrap.css"
)

for pattern in "${FORBIDDEN_PATTERNS[@]}"; do
    if [ -d "${UI_DIR}" ] && find "${UI_DIR}" -name "*${pattern}*" | grep -q .; then
        echo -e "${RED}[ERROR] Detected prohibited heavy bundle pattern in UI: ${pattern}${NC}"
        ERRORS=$((ERRORS + 1))
    fi
done

if [ "$ERRORS" -eq 0 ]; then
    echo -e "${GREEN}  ✓ Zero heavy/monolithic vendor frameworks detected.${NC}"
fi

echo -e "${BLUE}==> [3/3] Auditing Component References...${NC}"
if [ -d "${UI_DIR}/components" ]; then
    for component in "${UI_DIR}/components"/*.js; do
        if [ -f "$component" ]; then
            comp_name=$(basename "$component")
            # Check if referenced in index.html or other js
            if ! grep -q "$comp_name" "${UI_DIR}/index.html" "${UI_DIR}/js"/*.js 2>/dev/null; then
                echo -e "${YELLOW}[WARNING] Component '${comp_name}' is unreferenced in index.html or js modules.${NC}"
            else
                echo -e "${GREEN}  ✓ Component '${comp_name}' is actively referenced.${NC}"
            fi
        fi
    done
fi

if [ "$ERRORS" -gt 0 ]; then
    echo -e "${RED}FAILED: Frontend auditing failed with $ERRORS error(s).${NC}"
    exit 1
else
    echo -e "${GREEN}PASSED: Frontend is lightweight, clean, and debloated!${NC}"
    exit 0
fi
