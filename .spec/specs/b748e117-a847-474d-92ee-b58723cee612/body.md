# viewer-api: dev proxy

Canonical specification for `viewer-api::dev_proxy` — the optional
HTTP-reverse-proxy used during frontend development to forward unmatched
requests to a separately-running Trunk dev server.

## Public surface

- `dev_proxy::DevProxyConfig { upstream: Url, fallback_index: bool }`.
- `dev_proxy::dev_proxy_layer(config) -> impl Layer<…>` — proxies any
  request not matched by a previously registered route.
- CLI flag: `--dev[=URL]` activates the layer with a default upstream of
  `http://127.0.0.1:8080`.

## Demo behavior

The demo-viewer can be launched in two modes:

- **Production mode** (default): `viewer-ctl start demo-viewer` serves the
  pre-built `frontend/dioxus/dist/` directory.
- **Dev mode**: `cargo run -p demo-viewer -- --dev` proxies SPA assets to a
  separately-running `trunk serve` and continues to serve `/api/demo/*` from
  the Rust process.

The `pages/dev_proxy.rs` page renders:

1. The current mode (production / dev) and upstream URL.
2. A button that fetches `/__dev_probe` (a path only the dev server answers)
   and shows whether proxying is active.
3. Documentation of the recommended local dev workflow.

## Acceptance behavior (validated by e2e)

- In production mode, `/__dev_probe` returns `404`.
- In dev mode (e2e fixture spawns a tiny upstream that answers
  `/__dev_probe` with `pong`), the proxied response reaches the browser.
- API routes (`/api/demo/*`) are **always** served by the Rust process,
  never proxied.

## Code references

- `tools/viewer/viewer-api/src/dev_proxy.rs`
- `tools/viewer/e2e/tests/demo-viewer/dev-proxy.spec.ts`
