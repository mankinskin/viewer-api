# viewer-api: CodeViewer + FileContentViewer

Canonical specification for the shared code/file display Dioxus components
(`viewer-api/frontend/dioxus/src/components/code_viewer.rs` and
`file_content_viewer.rs`).

## Public surface

- `CodeViewer { source: String, language: Option<String>, start_line: u32,
  highlight: Vec<u32>, wrap: bool }` — line-numbered, syntax-coloured
  read-only code block.
- `FileContentViewer { path, range: Option<(u32,u32)>, on_load_error }` —
  fetches via `/api/demo/source?path=…&start=…&end=…` and renders through
  `CodeViewer`.

## Demo behavior

The `pages/code_viewer.rs` page demonstrates:

1. A static `CodeViewer` rendering ~30 lines of Rust with line 5 and 12
   highlighted.
2. A language switcher (Rust / TypeScript / Markdown / Plain) that re-renders
   the same buffer with different syntax highlighting.
3. A wrap/no-wrap toggle.
4. A `FileContentViewer` configured to load `viewer-api/src/auth.rs` lines
   1–40 from the demo backend.
5. Error handling: an invalid path that surfaces the error via `on_load_error`.

## Acceptance behavior (validated by e2e)

- Line numbers are rendered for every visible line, starting from `start_line`.
- Highlighted lines have a distinct background colour
  (computed style differs from non-highlighted lines).
- Toggling wrap changes `white-space` from `pre` to `pre-wrap`.
- The `FileContentViewer` populates within 2 s and shows `auth.rs` content.
- An unknown path triggers `on_load_error` and renders an error banner
  containing the status code.

## Code references

- `tools/viewer/viewer-api/frontend/dioxus/src/components/code_viewer.rs`
- `tools/viewer/viewer-api/frontend/dioxus/src/components/file_content_viewer.rs`
- `tools/viewer/e2e/tests/demo-viewer/code-viewer.spec.ts`
