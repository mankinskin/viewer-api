# viewer-ctl

`viewer-ctl` is the **lifecycle manager** for context-engine viewer servers,
their frontends, and the VS Code extensions that ship alongside them. It is a
single Rust binary, driven entirely by a TOML registry at the repository root.

It exists so that:

- A new viewer server, frontend, or extension can be registered by editing
  `viewer-ctl.toml` — no Rust code changes in `viewer-ctl` itself.
- Servers and their frontends have **independent** lifecycles; rebuilding a
  frontend does not require restarting the server it is served by, and
  installing a frontend does not require recompiling the server binary.
- All commands work uniformly across platforms, with explicit handling for
  Windows-specific concerns (file locks held by running servers, `.cmd`/`.bat`
  shims, locale-dependent `netstat` output).

---

## Goals

1. **Generic, config-driven** — the binary contains no hard-coded list of
   viewers, ports, or build commands. All knowledge of components lives in
   `viewer-ctl.toml`.
2. **Decoupled lifecycle** — servers, frontends, and extensions can each be
   built, installed, started, and stopped independently. A typical iterative
   loop is *edit → build frontend → install frontend* with the server
   continuously running.
3. **Single source of truth** — the same TOML file documents what exists,
   what builds what, and what serves what. `viewer-ctl list` prints the
   registry verbatim from config.
4. **No automatic restarts on Windows** — file locks held by a running server
   prevent atomic file replacement, but viewer-ctl never silently kills a
   server. The user opts in via the explicit `restart` subcommand.
5. **Predictable install layout** — every installed frontend lives at a fixed
   user-scoped path so servers can find it via a single `STATIC_DIR`
   environment variable.

---

## Non-Goals

- **Process supervision** — viewer-ctl does not keep servers alive, restart
  them on crash, or aggregate logs. Once `start` returns the server is
  detached (Windows) or replaces the viewer-ctl process (Unix `exec`).
- **Cross-process orchestration** — viewer-ctl manages one component per
  invocation. Multi-step pipelines belong in a `[[task]]` entry.
- **Build-system replacement** — viewer-ctl shells out to `cargo`, `trunk`,
  `npm`, `vsce`, and `code`. It is not a build cache or dependency manager.

---

## Architecture

```
viewer-ctl (binary)
└── src/
    ├── main.rs          — entry point + top-level dispatch
    ├── cli.rs           — clap CLI types (Cli, Cmd, KindArg)
    ├── logging.rs       — tagged stdout/stderr macros (info!/warn!/error!)
    ├── config.rs        — TOML loader for viewer-ctl.toml
    ├── paths.rs         — repo_root, copy_dir_contents, disp(&Path)
    ├── shell.rs         — run_cmd_*, which (Windows .cmd routing)
    ├── process.rs       — pids_on_port, kill_process, print_process_info
    └── commands/
        ├── mod.rs       — list/status + for_matching dispatch
        ├── server.rs    — build/install/start/stop + spawn semantics
        ├── frontend.rs  — build (with prebuild) + install (mirror to STATIC_DIR)
        ├── extension.rs — vscode build + install (sync or VSIX)
        └── task.rs      — multi-step shell pipeline
```

---

## Child Specs

This spec is the umbrella; concrete behaviour is described in child specs:

- `viewer-ctl/config` — the `viewer-ctl.toml` schema.
- `viewer-ctl/cli` — command-line interface contract.
- `viewer-ctl/lifecycle/server` — server build, install, start, stop.
- `viewer-ctl/lifecycle/frontend` — frontend build and install.
- `viewer-ctl/lifecycle/extension` — VS Code extension install.
- `viewer-ctl/lifecycle/task` — multi-step task runner.
- `viewer-ctl/process-management` — TCP-port discovery and termination.
- `viewer-ctl/install-layout` — on-disk layout of installed artifacts.
