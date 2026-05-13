## Problem

`viewer-api-dioxus` defines two parallel surface palettes in `public/css/variables.css`:

- Theme tokens (`--bg-primary`, `--bg-secondary`, `--bg-tertiary`, `--text-primary`, …) are **rewritten by the theme picker** (PAPER, SCRATCHBOARD, ARCADIA, DARK).
- Panel-surface tokens (`--panel-bg`, `--panel-bg-strong`, `--panel-bg-floor`) are **hardcoded dark translucent values** and never updated when the theme changes.

`.header`, `.sidebar`, and `.glass-panel` all use `--panel-bg*`. `.sidebar-header` and most form elements use `--bg-tertiary`. Result: with PAPER theme the user sees dark translucent panels with **dark text** on top, plus a single light-coloured sidebar header strip. Verified at runtime:

```
sidebar.background-color → rgba(20, 20, 24, 0.55)   // dark
sidebar.color            → rgb(44, 42, 38)          // dark too
sidebar-header bg        → #f2efe8                   // light
```

This is the most prominent visual defect in the screenshot at the top of the parent epic.

## Acceptance criteria

1. `--panel-bg`, `--panel-bg-strong`, `--panel-bg-floor` are **derived from the active theme**, not hardcoded.
   - Implementation: the `ThemeStore::apply_to_dom()` (or equivalent CSS-variable writer) writes `--panel-bg` family values from the theme palette (e.g. `mix(--bg-secondary-solid, transparent, panel_alpha)`), so PAPER yields a light translucent panel and DARK yields the current dark panel.
2. Background, text, and border tokens used by `.header`, `.sidebar`, `.sidebar-header`, `.sidebar-content`, `.sidebar-search`, and `.glass-panel` produce **WCAG AA contrast** under all four built-in themes (PAPER, SCRATCHBOARD, ARCADIA, DARK).
3. The four state-filter chips in the sidebar (`All` / `new` / `ready` / `impl` / `review` / `done` / `cancelled`) inherit theme-aware foreground and background — no hardcoded hex.
4. Manual screenshot verification at `http://localhost:3002` for each preset; attach before/after screenshots in the PR.
5. Playwright regression test under `tools/viewer/e2e/` that switches presets and asserts contrast on `.sidebar` and `.header`.

## Implementation notes

- Touchpoints: `tools/viewer/viewer-api/frontend/dioxus/src/store/theme.rs` (writer), `public/css/variables.css` (variable defaults), `public/css/layout.css` (consumers).
- `--panel-blur` and `--panel-saturate` should remain theme-independent (effect intensity).
- Keep solid fallbacks (`--bg-primary-solid` etc.) — they exist precisely for opaque modal use.

## Parent

Epic `4e2b2b0b-9f56-4786-991c-8f10e653f4c3`.
