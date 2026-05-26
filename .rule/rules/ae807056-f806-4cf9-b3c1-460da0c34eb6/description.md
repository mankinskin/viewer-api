## Supported commands

- `viewer-ctl start <viewer>` — boots the viewer in the background with the configured port and environment, holding a lock so only one instance runs.
- `viewer-ctl stop <viewer>` / `viewer-ctl restart <viewer>` — release the lock and rebuild as needed.
- `viewer-ctl prepare <viewer>` — builds the frontend assets the viewer needs before serving.
- `viewer-ctl status <viewer>` — reports running/idle plus the bound port.