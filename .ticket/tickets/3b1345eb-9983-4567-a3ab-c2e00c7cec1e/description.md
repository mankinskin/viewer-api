# viewer-ctl: add --log-dir and --log-level start flags

## Problem

There is no way to control the log directory or level for a viewer server launched via `viewer-ctl start` without modifying the binary's default env vars by hand. Operators cannot redirect logs to a custom path without code changes.

## Scope

Extend `viewer-ctl start` and `viewer-ctl restart` with two new optional flags:

```
viewer-ctl start <server> [--log-dir <path>] [--log-level <filter>]
```

Implement by injecting the corresponding env vars (`LOG_DIR`, `LOG_LEVEL`) into the spawned server process's environment before launch — no changes to the server binaries required, as all tools honour those env vars after [LOG-1a] and [LOG-1b] land.

### Changes

- `tools/viewer/viewer-ctl/src/cli.rs` — add `#[arg(long)] log_dir: Option<PathBuf>`, `#[arg(long)] log_level: Option<String>` to `Start` and `Restart`.
- `tools/viewer/viewer-ctl/src/commands/server.rs` — pass as additional `env_vars` to `spawn_server` / `run_server_foreground`.
- `viewer-ctl.toml` schema (optional) — allow `[server.env]` entries for `LOG_DIR` and `LOG_LEVEL` as persistent defaults per server.

## Acceptance Criteria

- `viewer-ctl start ticket-viewer --log-dir /tmp/my-logs` causes the server to write `ticket-viewer.log` under `/tmp/my-logs/`.
- `viewer-ctl start ticket-viewer --log-level debug` causes the server to log at DEBUG level.
- Without the flags, the server uses its compiled-in defaults (no regression).
- `viewer-ctl start ticket-viewer --fg --log-dir ./logs` works with the foreground flag from the prior viewer-ctl PR.

## Files

- `tools/viewer/viewer-ctl/src/cli.rs`
- `tools/viewer/viewer-ctl/src/commands/server.rs`

## Depends on

- [LOG-1a] and [LOG-1b] (servers must honour `LOG_DIR` / `LOG_LEVEL` env vars)
