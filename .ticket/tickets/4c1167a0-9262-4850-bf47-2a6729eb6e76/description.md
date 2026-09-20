# Port log-viewer app shell, URL state, and per-file store to Dioxus

## Problem

The current log-viewer frontend is not just a three-pane browser. `tools/viewer/log-viewer/frontend/src/App.tsx` coordinates the header, advanced filter panel, sidebar file tree, tabbed center pane, right-side code viewer, shared WGPU overlay, and per-file state restore from the URL. Existing migration tickets cover the Dioxus scaffold (`LOG-5a`) and a subset of the browser UI (`LOG-5b`), but they do not capture the state orchestration layer that makes the current frontend usable.

Without a dedicated migration ticket for that shell/state behavior, a Dioxus port will regress:
- URL restore for `/file/<name>/<tab?>`
- per-file cached state when switching files
- source/signature prefetch on file load
- advanced filters and JQ query flow
- FX/theme/header controls
- keyboard focus behavior across panels
- parity between live HTTP mode and static demo mode

## Scope

1. Port the top-level app shell from `tools/viewer/log-viewer/frontend/src/App.tsx` to Dioxus:
   - header
   - advanced filter panel
   - left sidebar file browser
   - tab bar / center content container
   - right-side code viewer panel
   - root overlay mount for viewer-api visual effects
2. Port the Preact signals store in `tools/viewer/log-viewer/frontend/src/store/index.ts` to a Dioxus store/service layer with equivalent semantics:
   - global state for `log_files`, `current_file`, `status_message`, loading/error flags
   - per-file cached state for entries, text search, JQ filter, level/type filters, selected entry, code viewer state, active tab, active search/path step, and signatures
   - file-load behavior that preloads signatures and an initial source location
3. Port URL/hash state handling currently implemented via `createUrlStateManager` so Dioxus restores and updates `/file/<name>/<tab?>` consistently.
4. Port the shared app controls currently exposed through the header/filter/sidebar:
   - text search submit / clear
   - refresh
   - advanced filter panel visibility
   - sidebar category filters and file tree selection
   - FX toggle and theme/settings entry point
5. Define the Dioxus equivalent of the current API mode split in `src/api/index.ts`:
   - preserve live HTTP mode
   - preserve static/demo mode, or explicitly replace it with a documented Dioxus build-time/runtime mechanism
6. Reuse `viewer-api-dioxus` primitives where possible. Extract missing reusable shell/state helpers into `viewer-api-dioxus` instead of re-implementing them only for log-viewer.

## Acceptance Criteria

- Opening the Dioxus app at `/file/<name>` or `/file/<name>/<tab>` restores the same file and tab as the current Preact frontend.
- Switching between files preserves each file's cached state for selected tab, search/JQ filter state, selection, and code viewer state.
- Header actions for search, clear, refresh, FX toggle, and theme/settings entry work in the Dioxus app.
- Sidebar category filters and file tree selection work against the current `/api/logs` backend.
- File load still prefetches signatures and an initial source file/line when available.
- The implementation lands in shared `viewer-api-dioxus` primitives where behavior is cross-viewer, not as log-viewer-only duplication.

## Validation

- `cargo check --target wasm32-unknown-unknown -p log-viewer-dioxus`
- `trunk serve` in `tools/viewer/log-viewer/frontend/dioxus`
- manual browser check for URL restore, file switching, search/clear, filter visibility, refresh, and theme/FX toggles

## Relevant Current Frontend Anchors

- `tools/viewer/log-viewer/frontend/src/App.tsx`
- `tools/viewer/log-viewer/frontend/src/store/index.ts`
- `tools/viewer/log-viewer/frontend/src/components/Header/Header.tsx`
- `tools/viewer/log-viewer/frontend/src/components/FilterPanel/FilterPanel.tsx`
- `tools/viewer/log-viewer/frontend/src/components/Sidebar/Sidebar.tsx`
- `tools/viewer/log-viewer/frontend/src/api/index.ts`
- `tools/viewer/log-viewer/frontend/src/api/live.ts`

## Depends on

- `LOG-5a` scaffold ticket
