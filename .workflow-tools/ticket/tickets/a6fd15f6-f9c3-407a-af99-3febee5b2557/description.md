**Optional / deferred.** Once the Dioxus shared crate is stable and proven in spec-viewer, expose its components to doc-viewer (Preact/TS) via thin TypeScript bindings or a wasm-bindgen surface, eliminating the duplicated implementations of breadcrumbs, tabs, cards, filter panel, etc.

## Why this is optional

doc-viewer works today and rewriting it carries no functional benefit. The motivation is purely deduplication. Defer until both:
1. P1–P5 are landed and stable for at least one cycle
2. A clear integration approach is chosen (TS bindings vs. mounting Dioxus components inside Preact vs. full Dioxus rewrite)

## Possible approaches

- **A. Stay in TS, copy-cat shared CSS only.** Cheapest. Already partially true (`viewer-api.css`).
- **B. wasm-bindgen wrapped Dioxus components.** Heavy; pulls dual runtimes into the page.
- **C. Full doc-viewer rewrite in Dioxus.** Largest change; cleanest end state.

## Acceptance criteria (when picked up)

- An ADR/design doc selects the integration strategy with rationale
- A pilot component (e.g., Breadcrumbs) is migrated end-to-end
- doc-viewer Playwright e2e suite continues to pass
