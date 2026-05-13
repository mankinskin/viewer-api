# viewer-api: source

Canonical specification for `viewer-api::source` — the safe source-file
serving helper that resolves a workspace-relative path against a configured
root, rejects path traversal, and streams text files with language hinting.

## Public surface

- `source::SourceRoots { roots: Vec<PathBuf> }`.
- `source::SourceQuery { path: String, start: Option<u32>, end: Option<u32> }`.
- `source::resolve(roots: &SourceRoots, path: &str) -> Result<PathBuf, ApiError>`
  — rejects absolute paths, `..` segments, and paths outside any root.
- `source::serve_file(roots, query) -> Result<SourceResponse, ApiError>`
  with `SourceResponse { path, language, lines: Vec<String>, start, end, total_lines }`.

## Demo behavior

The `pages/source.rs` page demonstrates:

1. A path input + start/end line inputs that hit `/api/demo/source?…`.
2. The response is rendered using the shared `CodeViewer` component, with
   the language detected from the file extension.
3. Pre-canned buttons for "viewer-api/lib.rs", "viewer-api/auth.rs", and
   a path-traversal probe (`../../../etc/passwd`) that proves the
   `403 Forbidden` mapping.

## Acceptance behavior (validated by e2e)

- `GET /api/demo/source?path=src/lib.rs` returns the file content.
- `GET /api/demo/source?path=src/lib.rs&start=10&end=20` returns lines 10–20
  inclusive, with the original 1-based line numbers preserved.
- `GET /api/demo/source?path=../../../etc/passwd` returns `403`.
- `GET /api/demo/source?path=does/not/exist` returns `404`.

## Code references

- `tools/viewer/viewer-api/src/source.rs`
- `tools/viewer/e2e/tests/demo-viewer/source.spec.ts`
