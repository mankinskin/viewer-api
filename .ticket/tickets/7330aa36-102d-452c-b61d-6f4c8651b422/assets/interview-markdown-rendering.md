# Interview: Markdown Rendering

**Date:** 2026-04-08
**Applies to:** `af19b0f6` (TicketContent viewer), `7330aa36` (CodeViewer/FileContentViewer)

## Question

TicketContent currently uses `marked` (JS) + `highlight.js` for markdown/code rendering.

- Should the Dioxus port use a pure-Rust stack (`pulldown-cmark` + `syntect`) compiled to WASM, or wrap existing JS libraries via web-sys for faster initial parity?
- How important is syntax highlighting quality? `syntect` supports fewer languages than `highlight.js` out of the box.

## Answer

**Use a pure-Rust stack. If we do not find a Rust alternative, we need to have a meeting. Basic syntax highlighting is acceptable.**

## Implications

- Use `pulldown-cmark` for Markdown → HTML conversion (mature, widely used)
- Use `syntect` for syntax highlighting (basic language coverage is acceptable)
- No JS library dependencies — full Rust/WASM pipeline
- If a critical rendering capability has no Rust equivalent, escalate to team meeting before falling back to JS
- Accept that some exotic languages may not have highlighting support initially
- Consider `comrak` as an alternative to `pulldown-cmark` (GFM-compatible)
