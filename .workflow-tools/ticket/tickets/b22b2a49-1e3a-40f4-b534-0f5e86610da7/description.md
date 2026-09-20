# Cut log-viewer over from Preact/Vite to Dioxus/trunk and lock in migration validation

## Problem

The current migration ticket set scaffolds and ports features, but it does not include the final cutover step: making the Dioxus frontend the default served build, preventing long-term drift between the old Preact app and the new Dioxus app, and defining the regression checks required before retiring the old surface.

Without an explicit cutover ticket, the repo can end up with two frontends that both partially work and neither remains authoritative.

## Scope

1. Switch the default log-viewer frontend integration to Dioxus/trunk in the places that matter for daily use:
   - `viewer-ctl prepare log-viewer`
   - `viewer-ctl start log-viewer`
   - workspace/build task wiring
2. Decide and implement the fate of the existing Preact frontend:
   - remove it, or
   - freeze it as a clearly non-authoritative reference/demo path with documented boundaries
3. Move or retain any required CSS/assets/demo/static-mode pieces so the Dioxus build is self-sufficient.
4. Ensure generated frontend build artifacts remain ignored in Git for the new Dioxus output path.
5. Add migration validation coverage for the flows the current frontend actually exercises:
   - file list and file selection
   - search/filter flows
   - source/code viewer flow
   - hypergraph tab
   - theme/settings + FX toggle
6. Update developer-facing docs or task descriptions so the Dioxus frontend is the documented source of truth.

## Acceptance Criteria

- `viewer-ctl prepare log-viewer` builds the Dioxus frontend and stages the correct static output.
- `viewer-ctl start log-viewer` serves the Dioxus frontend by default.
- The repository has one clearly documented authoritative log-viewer frontend implementation.
- Build artifacts for the selected Dioxus dist output are ignored by Git.
- Migration validation covers log browsing, source view, hypergraph, and theme/FX flows.
- Any remaining Preact frontend code is either removed or explicitly documented as non-authoritative.

## Validation

- `cargo check --target wasm32-unknown-unknown -p log-viewer-dioxus`
- `viewer-ctl prepare log-viewer`
- `viewer-ctl start log-viewer`
- smoke test / E2E coverage for the migrated Dioxus frontend flows

## Relevant Current Frontend Anchors

- `tools/viewer/log-viewer/frontend/package.json`
- `tools/viewer/log-viewer/frontend/src/main.tsx`
- `tools/viewer/log-viewer/frontend/src/App.tsx`
- `viewer-ctl.toml`
- any workspace tasks that still target the Preact build

## Depends on

- `LOG-5b` basic browser UI ticket
- `LOG-5e` visualization/overlay parity ticket
