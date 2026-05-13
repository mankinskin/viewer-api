# viewer-ctl wiring

- Add a `[viewers.demo-viewer]` entry to `viewer-ctl.toml` (port 3099,
  prepare + start commands matching the other viewers).
- Verify `viewer-ctl prepare demo-viewer`, `start`, `open`, `stop`
  all behave like the other managed viewers.
- Add a build task `demo-viewer: managed` and `demo-viewer: prepare`
  to `.vscode/tasks.json` mirroring the other viewer tasks.

## Acceptance

- `viewer-ctl status` lists `demo-viewer`.
- The VS Code Run Task picker shows the new tasks.
