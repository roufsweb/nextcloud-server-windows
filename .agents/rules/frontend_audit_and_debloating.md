# Frontend Component Auditing & Debloating Rules

This rule defines mandatory quality, size, and architecture constraints for the Nextcloud Server for Windows desktop GUI frontend.

---

## 1. Zero-Bloat & Under-500MB Size Budget

The complete distributed application (Rust supervisor + GUI frontend + Caddy + PHP 8.3 NTS + Cloudflared + Nextcloud) must strictly fit under the **500 MB total uncompressed budget** (and < 250 MB compressed zip).

### Frontend-Specific Budgets:
- **Total compiled UI bundle size**: `< 2.5 MB` (HTML + CSS + JS + SVGs combined).
- **Runtime memory footprint**: `< 40 MB` RAM for the desktop UI window.
- **Dependency discipline**: 
  - Never import monolithic component frameworks (e.g. huge bloated enterprise UI kits that bundle thousands of unused components).
  - Use modular, atomic, tree-shakable components.
  - Zero heavy runtime frameworks that require multi-megabyte runtimes.

---

## 2. Component Auditing & Dead Code Pruning

Before packaging or merging frontend changes:
1. **Unused Component Elimination**:
   - Any component file (e.g. `StorageGauge.svelte`, `DriveCard.js`, etc.) that is not actively mounted or routed in the UI must be deleted immediately.
   - Dead imports and commented-out experimental components must not be checked into VCS.
2. **Icon & Asset Auditing**:
   - Never import entire icon packs (e.g. FontAwesome full 10MB webfonts).
   - Embed only the exact inline SVG icons required for the application.
3. **CSS Tree-Shaking**:
   - Purge all unused CSS classes in production builds.
   - Use CSS variables for theme switching (dark/light mode) instead of duplicating stylesheets.

---

## 3. Storage & Drive UI Requirements

The frontend must provide an intuitive, error-proof storage management experience:
- **Data Root Selector**:
  - Clear visual indicator of the active Nextcloud Data root directory.
  - One-click folder picker and path validator.
- **On-Demand Drive Mounting**:
  - Dedicated "Add Drive / External Storage" modal/card.
  - Automatic NTFS permission check with real-time feedback (e.g. "Granting Full Access via icacls...").
  - Instant display of drive capacity (Used / Free / Total GB) with visual progress bars.
- **In-App Logs Dashboard**:
  - High-performance virtualized or ring-buffered log console (prevents UI lag when processing thousands of log lines).
  - Filter tabs: `All`, `Caddy`, `PHP`, `Nextcloud Core`, `Cloudflared`.
  - Pause stream, clear buffer, and copy to clipboard buttons.
