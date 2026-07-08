<!-- aligned-structure:v1 -->

# Summary

viewer-ctl installs three classes of artifacts. The layout is fixed and documented so that downstream tools (servers, scripts, documentation) can rely on knowing where things land.

## Behavior Story

viewer-ctl installs three classes of artifacts. The layout is fixed and documented so that downstream tools (servers, scripts, documentation) can rely on knowing where things land.

## Provided Surface Contracts

- Define provided contracts for this behavior slice.

## Required Validation

- Triangulate behavior with executable checks, natural-language clauses, and code/schema/API references when available.

## Related Implementation Tickets

- No related implementation ticket is linked yet.

## Background Knowledge References

- Prefer entity references and context rendering over embedding fully expanded payloads in this spec body.

## Legacy Content (Preserved)

# viewer-ctl/install-layout

viewer-ctl installs three classes of artifacts. The layout is fixed and
documented so that downstream tools (servers, scripts, documentation) can
rely on knowing where things land.

---

## Server binaries

Installed via `cargo install --path <source_dir> --force` into Cargo's
default binary directory:

```text
$CARGO_HOME/bin/<package>            ($CARGO_HOME defaults to ~/.cargo)
```

The user's `$PATH` must contain `$CARGO_HOME/bin` for `viewer-ctl start`
to locate the binary via `which`/`where`. If the lookup fails, viewer-ctl
runs `install_server` once and retries.

---

## Frontend bundles

Each frontend installs to its own subdirectory under
`[defaults].frontend_install_root`:

```text
<frontend_install_root>/<frontend.name>/
├── index.html
├── *.wasm   *.js   *.css   (hashed bundles)
└── …extra_assets, copied verbatim…
```

For the default `frontend_install_root = "~/.context-engine/static"`:

```text
~/.context-engine/static/spec-viewer/index.html
~/.context-engine/static/ticket-viewer/index.html
~/.context-engine/static/doc-viewer/index.html
~/.context-engine/static/log-viewer/index.html
```

`viewer-ctl start <server>` automatically passes the matching frontend's
install dir as `STATIC_DIR=<...>` if a `[[frontend]]` declares
`serves = "<server>"` *and* the install dir exists. If only one of those
is true, the server is launched without `STATIC_DIR` and falls back to
its compile-time default.

The install step always wipes and recreates this directory; stale hashed
bundles never persist across installs.

---

## VS Code extensions

Installed under VS Code's per-user extension directory:

```text
$USERPROFILE/.vscode/extensions/<publisher>.<name>-<version>/   (Windows)
$HOME/.vscode/extensions/<publisher>.<name>-<version>/          (Unix)
```

The fast install path mirrors `out/`, `resources/`, `node_modules/` and
the package manifest into this directory. A VS Code window reload is
required to activate the new code; viewer-ctl prints this reminder
after every install.

---

## Acceptance Criteria

- `viewer-ctl list` prints the resolved install path for every frontend so
  the user can see exactly where bundles will land.
- A leading `~` in `frontend_install_root` is always expanded; the literal
  string `~` never reaches a filesystem call.
- Re-installing a frontend never leaves stale files in the install dir.
- A server launched by `viewer-ctl start` always sees `STATIC_DIR` when
  both the linkage and the install dir exist.
