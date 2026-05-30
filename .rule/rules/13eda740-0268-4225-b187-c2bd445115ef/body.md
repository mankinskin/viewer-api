## Why a single boundary

- Port allocation, locking, and environment composition live in one place; ad-hoc commands bypass them and collide with each other.
- VS Code tasks (`*: managed start`, `*: open`) delegate to `viewer-ctl` so the editor and the CLI share the same lifecycle view.
- Health checks, log capture, and browser-validation flows all assume the viewer was started through `viewer-ctl`. Bypassing it produces stale process state, mismatched ports, and silently broken E2E runs.