# viewer-ctl/lifecycle/extension

VS Code extensions are TypeScript projects compiled to `out/`. viewer-ctl
provides build and install steps; the runtime side is owned by VS Code
itself.

---

## `build_extension`

Run `build_cmd` from `source_dir`. For typical extensions this is
`npm run compile`, which transpiles TypeScript into `out/`.

## `install_extension`

`install_extension` always runs `build_extension` first to guarantee a
fresh `out/` directory, then dispatches on `kind`:

- `kind = "vscode"` → `install_vscode_extension` (described below)
- any other kind → hard error (`unknown extension kind`)

---

## `install_vscode_extension`

The installer chooses one of two paths depending on whether the extension
is already present in the user's profile.

### Identifier resolution

Read `<source_dir>/<package_json>` and parse `publisher`, `name`, `version`.
Compose `dirname = "<publisher>.<name>-<version>"`. When `publisher` is
missing, `"undefined_publisher"` is used (matches `vsce`'s convention).

The target directory is:

```text
$USERPROFILE/.vscode/extensions/<dirname>/      (Windows)
$HOME/.vscode/extensions/<dirname>/             (Unix)
```

When neither variable is set, installation aborts with an explicit error.

### Fast path (sync) — extension dir already exists

Mirror the build output into the existing install dir:

1. Wipe `<install_dir>/out/` and copy `<source_dir>/out/` into it.
2. Copy `<source_dir>/resources/` to `<install_dir>/resources/` if it exists.
3. Overwrite `<install_dir>/<package_json>` with the source manifest.
4. Mirror `<source_dir>/node_modules/` if present.

This avoids a full repackage round-trip during iteration, which on large
extensions can take many seconds. The user must reload the VS Code window
for the changes to activate.

### Slow path (first install) — VSIX

1. `vsce package --no-dependencies --allow-missing-repository --skip-license`
   in `source_dir`. This produces `<name>-<version>.vsix`.
2. Find the newest `*.vsix` in `source_dir` (modification time).
3. `code --install-extension <path-to-vsix> --force`.

The VSIX path is intentionally not cleaned up; it remains in the source
directory and is overwritten on subsequent first installs.

---

## Acceptance Criteria

- A first-time install on a machine without `vsce` errors out cleanly with
  the missing-binary message instead of partial state.
- A subsequent install always uses the fast path so iteration stays sub-second
  for the common case (`out/` only).
- The user is reminded after both paths that a VS Code window reload is
  required to activate the new code.
