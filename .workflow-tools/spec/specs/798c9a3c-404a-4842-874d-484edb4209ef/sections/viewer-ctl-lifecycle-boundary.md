# viewer-ctl lifecycle boundary

`viewer-ctl` is the single supported lifecycle boundary for every managed viewer. Starting, stopping, restarting, preparing, and inspecting a viewer must go through `viewer-ctl` rather than through ad-hoc `cargo run`, raw `trunk serve`, or hand-rolled scripts.

## Supported commands

- `viewer-ctl start <viewer>` — boots the viewer in the background with the configured port and environment, holding a lock so only one instance runs.
- `viewer-ctl stop <viewer>` / `viewer-ctl restart <viewer>` — release the lock and rebuild as needed.
- `viewer-ctl prepare <viewer>` — builds the frontend assets the viewer needs before serving.
- `viewer-ctl status <viewer>` — reports running/idle plus the bound port.

## Why a single boundary

- Port allocation, locking, and environment composition live in one place; ad-hoc commands bypass them and collide with each other.
- VS Code tasks (`*: managed start`, `*: open`) delegate to `viewer-ctl` so the editor and the CLI share the same lifecycle view.
- Health checks, log capture, and browser-validation flows all assume the viewer was started through `viewer-ctl`. Bypassing it produces stale process state, mismatched ports, and silently broken E2E runs.

## Constraints

- Never invoke `cargo run --bin <viewer>` directly for day-to-day work. Use `viewer-ctl` so the lock and the environment match what tests and the editor expect.
- `trunk serve` is used only by the lowest-level frontend dev task (`context-editor: trunk serve`) and is itself wrapped by VS Code tasks; other viewers go through `viewer-ctl prepare` + `viewer-ctl start`.
