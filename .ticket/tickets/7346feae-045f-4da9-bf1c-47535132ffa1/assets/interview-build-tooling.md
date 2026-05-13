# Interview: Build Tooling

**Date:** 2026-04-08
**Applies to:** `7346feae` (viewer-api-dioxus scaffold), `44d22e8f` (ticket-viewer scaffold)

## Question

context-editor uses `trunk serve` for dev. Options: Trunk (proven) or `dx serve` (Dioxus CLI, hot-reload)?

## Answer

**Let us start with the Dioxus-integrated `dx serve`.**

## Implications

- Use `dx serve` instead of `trunk serve` for development
- Requires `dioxus-cli` installed (`cargo install dioxus-cli`)
- Configuration via `Dioxus.toml` instead of `Trunk.toml`
- Hot-reload support (Dioxus CLI provides component-level hot-reload for RSX)
- Diverges from context-editor's Trunk setup — but aligns with Dioxus ecosystem
- May need `dx build --release` for production WASM builds
- Verify `dx serve` works well with web-sys/WebGPU features before committing
- Falls back to Trunk if `dx serve` proves unstable or limiting
