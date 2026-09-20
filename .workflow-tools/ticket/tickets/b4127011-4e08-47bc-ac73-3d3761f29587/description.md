Port doc-viewer's filter panel (basic dropdown filters + JQ presets + custom JQ input + results list) to a generic Dioxus shell.

## Deliverables

1. `components/filter_panel.rs`
   - Generic shell: `FilterPanel { state: Signal<FilterState>, presets: Vec<FilterPreset>, on_query_change: EventHandler<String>, results: Option<Vec<FilterResult>>, on_result_click: EventHandler<String>, loading: bool }`
   - `FilterState` carries arbitrary key/value pairs; viewer maps its filter dropdowns onto them
   - `FilterPreset { label, jq }`, `FilterResult { id, title, summary }`
   - Collapsible header, two-column layout (basic / advanced), preset chips, custom JQ form, result list
   - CSS: `filter-panel.css` ported from `tools/viewer/doc-viewer/frontend/src/styles/filter-panel.css`, classes namespaced `.filter-panel__*`

2. `components/header.rs` integration
   - `HeaderActions` (P3) gains optional `filter_panel_open` signal so the toggle button reflects state

## JQ backend (per-viewer)

- This ticket ships only the UI shell. Each consuming viewer must wire its own backend that takes a JQ string and returns `Vec<FilterResult>`.
- For spec-viewer: spike a `/specs/jq` endpoint (separate ticket if non-trivial); fall back to client-side filtering of an in-memory list if the backend is too costly to add in this ticket.

## Acceptance criteria

- `cargo check -p viewer-api-dioxus --target wasm32-unknown-unknown` passes
- A spec-viewer demo route or the spec-list page wires the panel against a stub backend (mocked results) and toggling/clearing/preset selection works in the browser
- CSS uses tokens; no hardcoded colours

## Risk / scope notes

- Server-side jq evaluation in spec-viewer is not in scope here. If client-side filtering is chosen, document that in the PR.
- If TabsStore (P2) is integrated, clicking a result opens the spec in a new tab.
