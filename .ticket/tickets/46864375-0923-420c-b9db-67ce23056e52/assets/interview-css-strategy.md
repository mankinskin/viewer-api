# Interview: CSS Strategy

**Date:** 2026-04-08
**Applies to:** `2405a83e` (CSS stylesheets port), `46864375` (Theme system)

## Question

The current viewer-api uses 9 CSS files with CSS custom properties (40+ `--theme-*` variables). context-editor uses Tailwind CDN classes.

- Do you want the Dioxus port to keep the CSS custom properties system (easier 1:1 port, runtime theme switching) or migrate to Tailwind (consistent with context-editor, utility-first)?
- Or a hybrid: Tailwind for layout + CSS variables for theme colors?

## Answer

**Keep custom properties — 1:1 port.**

## Implications

- Port all 9 CSS files from `viewer-api/frontend/` directly
- Preserve the `--theme-*` custom property system for runtime theme switching
- ThemeStore applies themes by setting CSS variables on `:root`
- No Tailwind dependency (diverges from context-editor approach)
- 4 theme presets (Arcadia, Dark, Paper, Scratchboard) port as-is
