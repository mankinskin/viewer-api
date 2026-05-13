# viewer-ctl/cli

The viewer-ctl command-line interface is defined by `clap` in `src/cli.rs`.
This document is the contract: **the surface defined here is stable** and
must not be broken without bumping the binary's compatibility expectations.

---

## Synopsis

```text
viewer-ctl <subcommand> [args...]
```

All subcommands return exit code `0` on success and `1` on failure, with the
error message printed to stderr as `error: <reason>`.

---

## Subcommands

### `list`

Print every component in the registry. Read-only; never touches the
filesystem outside `viewer-ctl.toml`.

### `status [<name>]`

For each server (or just the named one), report whether a process is
listening on its configured port. Output is plain-text, one server per line.

### `build <name> [--kind <server|frontend|extension>]`

Build artifacts for the matching component(s). Per kind:

- **server**   → `cargo build --release --manifest-path <source_dir>/Cargo.toml`
- **frontend** → run `[[frontend.prebuild]]` steps, then `build_cmd`,
  then verify `build_output/index.html` exists, then mirror `extra_assets`.
- **extension** → execute `build_cmd` from `source_dir`.

### `install <name> [--kind <server|frontend|extension>]`

Install previously-built artifacts. Per kind:

- **server**   → `cargo install --path <source_dir> --force`
- **frontend** → wipe `<install_root>/<name>/`, recreate it, copy
  `build_output/` contents into it. Requires a successful prior `build`.
- **extension** → kind-specific (currently `vscode`); also runs `build_cmd`
  first to ensure `out/` is fresh.

### `start <server> [-- <extra-args>...]`

Launch a server. Sequence:

1. If the configured port is occupied, kill the listener(s).
2. Resolve the server binary via `which`/`where`. If absent, run
   `install --kind server` once and retry.
3. Compose env: copy `[[server]].env`, append `STATIC_DIR=<install_root>/<linked frontend>`
   when that directory exists, append `PORT=<port>`.
4. Append `[[server]].start_args` followed by any extra CLI args after `--`.
5. On Unix, `exec()` the binary (replacing viewer-ctl).
   On Windows, spawn detached with stdio redirected to NUL and exit
   immediately so viewer-ctl releases any file handles.

### `stop <server>`

Find PIDs listening on the server's port and terminate them. Prints
identifying info (image name / args) for each PID before the kill attempt.
Returns failure if any PID resists termination.

### `restart <server> [-- <extra-args>...]`

`stop`, sleep 500 ms, `start`. Never invoked implicitly anywhere else.

### `task <name>`

Execute every `[[task.steps]]` entry in order. A step's failure stops the
task unless its `allow_failure = true`.

---

## Flag conventions

- Component name is always positional and required.
- `--kind` filters dispatch when a name is shared across kinds.
  Without `--kind`, all matching components are processed in the order
  *server → frontend → extension*.
- Extra server arguments use the standard clap `--` sentinel so they are
  passed verbatim, including arguments starting with `-`.

---

## Error model

Every command returns `Result<(), String>` internally. The string is
printed verbatim after `error: ` and the process exits with `1`. There are
no panics on the happy or failure paths.
