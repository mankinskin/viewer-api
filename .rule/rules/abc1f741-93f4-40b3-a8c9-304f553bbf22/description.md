## Tool Use Examples

```bash
cargo run -p viewer-ctl -- list
cargo run -p viewer-ctl -- start spec-viewer
cargo check -p viewer-api
cargo check --target wasm32-unknown-unknown -p viewer-api-dioxus
```

- Inspect which viewer components are declared in `viewer-ctl.toml`.
- Start a configured viewer through the lifecycle manager.
- Validate the shared backend runtime in `viewer-api`.
- Validate the shared browser scaffold in `viewer-api-dioxus`.
