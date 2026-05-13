# viewer-ctl/config

The component registry that drives viewer-ctl is a single TOML file at the
repository root: `viewer-ctl.toml`. It is the **only** input that
distinguishes one component from another; the binary contains no hard-coded
component identifiers.

---

## Discovery

`viewer-ctl` locates the registry by walking ancestors of two starting
points and selecting the first directory that contains `viewer-ctl.toml`:

1. `$CARGO_MANIFEST_DIR` (set automatically by `cargo run`).
2. The current working directory.

If neither walk finds a registry, the cwd is used as-is and component
lookup will fail with a clear error.

---

## Top-Level Sections

```toml
[defaults]
frontend_install_root = "~/.context-engine/static"

[[server]]    ...   # repeatable
[[frontend]]  ...   # repeatable
[[extension]] ...   # repeatable
[[task]]      ...   # repeatable
```

A leading `~` in any path is expanded to the user's home directory
(`$USERPROFILE` on Windows, `$HOME` elsewhere) via the `dirs` crate.

---

## `[defaults]`

| Field                  | Type   | Default                       | Purpose                                                                 |
|------------------------|--------|-------------------------------|-------------------------------------------------------------------------|
| `frontend_install_root`| string | `"~/.context-engine/static"`  | Parent directory under which each installed frontend gets a sub-folder. |

---

## `[[server]]`

| Field        | Type            | Required | Purpose                                                              |
|--------------|-----------------|----------|----------------------------------------------------------------------|
| `name`       | string          | yes      | Lookup key used by every CLI command.                                |
| `package`    | string          | yes      | Cargo package + binary name. Resolved via `which` (or `where`).      |
| `port`       | u16             | yes      | Default port, overridable at runtime by `$PORT`.                     |
| `source_dir` | string (path)   | yes      | Crate directory relative to the repo root.                           |
| `start_args` | list of strings | no       | Always-prepended args passed to the binary on `start`.               |
| `env`        | map<str, str>   | no       | Always-set environment variables for the launched server.            |

---

## `[[frontend]]`

| Field          | Type            | Required | Purpose                                                                                  |
|----------------|-----------------|----------|------------------------------------------------------------------------------------------|
| `name`         | string          | yes      | Lookup key. May intentionally match a `[[server]].name` (paired components).             |
| `serves`       | string          | no       | Name of a `[[server]]` this frontend is served by. Validated at load time.               |
| `source_dir`   | string (path)   | yes      | Frontend source root, relative to repo root.                                             |
| `build_cmd`    | list of strings | yes      | Command run from `source_dir`, e.g. `["trunk", "build", "--release"]`.                   |
| `build_output` | string (path)   | yes      | Directory containing the built `index.html` after `build_cmd` succeeds.                  |
| `extra_assets` | list of paths   | no       | Directories whose contents are copied **into** `build_output` after the build.           |
| `prebuild`     | list of step    | no       | Optional preparatory steps; see schema below.                                            |

**Validation:** Every `serves` value must reference a known `[[server]].name`,
otherwise `Config::load` returns an error and viewer-ctl aborts.

### `[[frontend.prebuild]]`

| Field       | Type            | Required | Purpose                                                                              |
|-------------|-----------------|----------|--------------------------------------------------------------------------------------|
| `dir`       | string (path)   | yes      | Working directory for the step.                                                      |
| `cmd`       | list of strings | yes      | Command to execute.                                                                  |
| `condition` | string          | no       | Skip predicate. Currently supported: `"missing:<relpath>"` runs only if absent.      |

Unknown conditions fail open (the step runs).

---

## `[[extension]]`

| Field          | Type            | Required | Purpose                                                                              |
|----------------|-----------------|----------|--------------------------------------------------------------------------------------|
| `name`         | string          | yes      | Lookup key.                                                                          |
| `kind`         | string          | yes      | Installer kind. Currently only `"vscode"` is recognised.                             |
| `source_dir`   | string (path)   | yes      | Extension source root.                                                               |
| `package_json` | string          | no       | Manifest filename, default `"package.json"`. Must exist and be valid JSON.           |
| `build_cmd`    | list of strings | yes      | e.g. `["npm", "run", "compile"]`.                                                    |

---

## `[[task]]`

| Field         | Type   | Required | Purpose                                                  |
|---------------|--------|----------|----------------------------------------------------------|
| `name`        | string | yes      | Lookup key.                                              |
| `description` | string | no       | Printed before the steps run.                            |
| `steps`       | list   | no       | Ordered list of `[[task.steps]]`; see below.             |

### `[[task.steps]]`

| Field           | Type            | Required | Purpose                                                |
|-----------------|-----------------|----------|--------------------------------------------------------|
| `dir`           | string (path)   | yes      | Working directory, relative to repo root.              |
| `cmd`           | list of strings | yes      | Command to execute.                                    |
| `allow_failure` | bool            | no       | If true, a non-zero exit logs a warning and continues. |
