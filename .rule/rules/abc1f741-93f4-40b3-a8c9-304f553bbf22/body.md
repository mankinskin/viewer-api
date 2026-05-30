## Tool Use Examples

### Install the viewer toolchain

From the `context-engine` repo root, install the lifecycle manager and WASM frontend builder together:

```bash
bash ./install-tools.sh --tool viewer-ctl --tool trunk
```

If you are working from a standalone `memory-viewers/viewer-api` checkout, use Cargo directly from this workspace:

```bash
cargo install --path viewer-ctl --bin viewer-ctl
cargo install trunk
```

After that, `viewer-ctl prepare ...` can build the Dioxus frontends because the `trunk` command is on `PATH`.

```bash
viewer-ctl list
viewer-ctl start spec-viewer
cargo check -p viewer-api
cargo check --target wasm32-unknown-unknown -p viewer-api-dioxus
```

- Inspect which viewer components are declared in `viewer-ctl.toml`.
- Start a configured viewer through the lifecycle manager.
- Validate the shared backend runtime in `viewer-api`.
- Validate the shared browser scaffold in `viewer-api-dioxus`.
