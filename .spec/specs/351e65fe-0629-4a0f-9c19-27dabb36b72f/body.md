# viewer-ctl/lifecycle/server

Servers are long-running Rust binaries that bind a TCP port. viewer-ctl owns
their build, install, start, and stop steps; once started, the server is no
longer supervised.

---

## `build_server`

```text
cargo build --release --manifest-path <source_dir>/Cargo.toml
```

Run from the repo root. Produces no installed artifact; it only warms the
target/ cache and is a sanity check.

## `install_server`

```text
cargo install --path <source_dir> --force
```

Installs the binary into `~/.cargo/bin/` (Cargo's default). `--force`
ensures a stale binary cannot block the install. Subsequent `start`
invocations resolve the binary via `which <package>` (or `where` on
Windows).

---

## `start <server> [-- extra-args...]`

Sequence is strict and side-effecting:

1. **Resolve port.** `port = $PORT.parse().unwrap_or(server.port)`.
2. **Free port.** Call `pids_on_port(port)`. If non-empty, kill each PID
   (see `viewer-ctl/process-management`), wait 1 s, re-check. If still
   occupied → abort with error.
3. **Resolve binary.** Try `which(package)`. On failure, run
   `install_server` once and retry. A second failure aborts with a
   PATH-related error.
4. **Compose environment.**
   - Start with `[[server]].env`.
   - If a `[[frontend]]` declares `serves = "<server>"` *and* the resolved
     install dir `<install_root>/<frontend.name>/` exists, append
     `STATIC_DIR=<that path>`.
   - Append `PORT=<port>`.
5. **Compose argv.** `[[server]].start_args ++ extra-args`.
6. **Launch.** Platform-dependent:
   - **Unix:** `Command::exec()` — replaces the viewer-ctl process image.
     If exec returns, the call necessarily failed and an error is bubbled.
   - **Windows:** `Command::spawn()` with `stdin/stdout/stderr → Stdio::null()`,
     then return immediately. viewer-ctl exits, releasing any file handles
     it held on `viewer-ctl.toml` or build outputs.

`STATIC_DIR` is the only environment variable wired through configuration;
viewer servers are expected to read it on boot and fall back to a
compile-time default if absent.

---

## `stop <server>`

1. Compute port (same `$PORT` override as `start`).
2. List PIDs via `pids_on_port`.
3. For each PID, call `print_process_info` (best-effort identifying line),
   then `kill_process`.
4. If any kill returns false, print a manual recovery hint
   (`taskkill /F /PID <pid>` on Windows, `kill -9 <pid>` elsewhere) and
   abort with error.

`stop` is idempotent: a port with no listeners exits successfully with a
single informational line.

---

## Acceptance Criteria

- `start` never silently kills a server it did not just launch unless that
  server is bound to the configured port.
- `start` works on a clean machine: a missing binary triggers exactly one
  recovery `install`, no infinite loop.
- `stop` exits 0 when nothing is listening on the port.
- On Windows, after `start` returns, no file under the repository tree
  remains locked by viewer-ctl itself.
- The combination `STATIC_DIR=<linked-frontend install dir>, PORT=<port>` is
  always present in the launched server's environment when both inputs are
  configured and present on disk.
