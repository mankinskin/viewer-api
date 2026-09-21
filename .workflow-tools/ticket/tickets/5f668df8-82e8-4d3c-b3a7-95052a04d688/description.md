Root cause: the Dioxus Trunk entrypoints for ticket-viewer, spec-viewer, and viewer-api omitted the shared `modal.css` bundle, so the Theme Settings overlay mounted without the fixed backdrop and stacking rules. The panel existed in the DOM, but the overlay styling degraded to static layout.

Implemented:
- Added `modal.css` to the shared CSS bundle list in the ticket-viewer, spec-viewer, and viewer-api Dioxus `index.html` entrypoints.
- Strengthened the focused ticket-viewer and spec-viewer Playwright regressions to assert fixed modal backdrop styling and attach a `.theme-settings.glass-panel` screenshot for visual review.
- Updated the canonical and viewer-specific theme-settings specs with the visible modal overlay contract and the focused validation commands.

Validation:
- `npm run test:e2e:release -- e2e-release/ticket-viewer.release.spec.ts -g "theme settings palette button opens and closes the theme settings panel"` in `memory-viewers/ticket-viewer/frontend/dioxus`
- `PLAYWRIGHT_REUSE_SERVER=1 npm run test:e2e:release -- e2e-release/viewer-api-primitives.spec.ts -g "P5.4 Overlay: theme settings open in a role=dialog modal-backdrop"` in `memory-viewers/spec-viewer/frontend/dioxus`
- Live screenshot verification on the refreshed managed viewers at `http://127.0.0.1:3002/` and `http://127.0.0.1:4002/specs`, confirming the visible modal and dimmed backdrop.