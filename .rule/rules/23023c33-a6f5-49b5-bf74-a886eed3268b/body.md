## Constraints

- Never invoke `cargo run --bin <viewer>` directly for day-to-day work. Use `viewer-ctl` so the lock and the environment match what tests and the editor expect.
- `trunk serve` is used only by the lowest-level frontend dev task (`context-editor: trunk serve`) and is itself wrapped by VS Code tasks; other viewers go through `viewer-ctl prepare` + `viewer-ctl start`.