# viewer-ctl/lifecycle/frontend

Frontends are static-asset bundles produced by `trunk` (Dioxus/WASM) or
`npx vite` (TypeScript SPAs). viewer-ctl treats them as opaque directories
that must contain `index.html` after a successful build.

---

## `build_frontend`

1. Verify `source_dir` exists; abort with a clear path-printed error
   otherwise.
2. **Prebuild.** For each `[[frontend.prebuild]]`:
   - Resolve `dir` relative to repo root.
   - Evaluate `condition`. The only currently-supported predicate is
     `missing:<relpath>` — the step runs **only** when `<dir>/<relpath>`
     does not exist on disk. Unknown conditions fail open (step runs).
   - Run `cmd` from `dir`.
3. **Build.** Run `build_cmd` from `source_dir`.
4. **Verify.** `build_output/index.html` must exist after the build.
   Absence is a hard error.
5. **Merge extra assets.** For each `extra_assets` entry, recursively
   copy the contents into `build_output`. This consolidates any files
   the build tool skipped (e.g. trunk's `public/` static directory) so
   `install_frontend` only needs to mirror a single directory.

---

## `install_frontend`

1. Confirm `build_output/index.html` exists; if not, instruct the user to
   run `build` first and abort.
2. Resolve the install dir: `<frontend_install_root>/<frontend.name>/`.
3. **Wipe and recreate** the install dir. This two-step approach guarantees
   stale hashed bundles (`*.wasm`, `*.js`) from previous builds cannot
   linger and confuse the running server.
4. Recursively copy `build_output/` into the install dir.

After install, the linked server will pick up the new files on its next
read of `STATIC_DIR`. On Windows, a server that holds a file handle on
`index.html` will keep serving the old version until restarted; viewer-ctl
deliberately does **not** initiate that restart.

---

## Build/Install Decoupling

Frontends and servers are independent components. The intended development
loop is:

```bash
# One-time setup
viewer-ctl install spec-viewer --kind server     # cargo install
viewer-ctl install spec-viewer --kind frontend   # publish initial bundle
viewer-ctl start spec-viewer                     # launch (detached)

# Iterative loop
edit frontend code…
viewer-ctl build   spec-viewer --kind frontend
viewer-ctl install spec-viewer --kind frontend
# server keeps running; refresh the browser to see changes
# (on Windows, restart the server when index.html lock blocks updates)
```

When a server and frontend share a name and `--kind` is omitted, both are
processed (server first, then frontend) — useful for first-time setup but
typically *not* what you want during iteration.

---

## Acceptance Criteria

- A frontend with no `prebuild` array installs cleanly on a fresh checkout
  provided its build dependencies are present (npm/trunk).
- `install_frontend` is atomic from the user's perspective: the install
  dir either contains a complete bundle from the most recent build or is
  empty (during the wipe step). It never contains a half-merged mix.
- A `prebuild` step with `condition = "missing:node_modules"` runs once
  on a fresh checkout and is skipped on every subsequent build.
- Servers that read `STATIC_DIR` on boot pick up newly-installed bundles
  on their next start without re-installation of the server binary.
