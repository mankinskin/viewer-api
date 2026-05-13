# viewer-api: demo-viewer

The **demo-viewer** is a reference application that lives inside the `viewer-api`
crate (`tools/viewer/viewer-api/examples/demo-viewer/`) and exists for one
purpose: **showcase, exercise, and end-to-end-test every public feature of
`viewer-api`** in a single, hosted application.

It is intentionally not a "product" viewer — it is the canonical integration
surface for the shared infrastructure that `log-viewer`, `doc-viewer`,
`ticket-viewer` and `spec-viewer` are built on. When a new feature is added to
`viewer-api`, a corresponding section of the demo-viewer must be added (or
extended) and an e2e test must be linked to the relevant feature spec.

## Goals

1. **Discoverability** — every public surface of `viewer-api` is reachable from
   the demo-viewer's left navigation, organised by feature group.
2. **Executable specification** — each demo page demonstrates the feature in
   the **canonical** way that downstream viewers should mimic.
3. **Test bed** — every demo page has at least one Playwright e2e test under
   `tools/viewer/e2e/tests/demo-viewer/` that exercises the feature in a real
   Chromium browser; the test is referenced from the corresponding feature spec.
4. **Regression detection** — running the demo-viewer e2e suite catches
   breaking changes in any shared primitive before they reach the product
   viewers.

## Non-goals

- Not a replacement for unit or integration tests of individual modules.
- Not a "playground" for unstable APIs — only stable, public surface.
- Not a production-grade application; auth, persistence and styling are
  minimal (whatever is needed to demonstrate the feature).

## Topology

```
tools/viewer/viewer-api/examples/demo-viewer/
├── Cargo.toml                    # bin = demo-viewer, depends on viewer-api
├── README.md                     # how to run + which features are showcased
├── src/main.rs                   # ServerConfig + run_server + routes
└── frontend/dioxus/              # Dioxus SPA mirroring ticket-viewer layout
    ├── Cargo.toml
    ├── Trunk.toml
    ├── index.html
    ├── public/                   # viewer-api.css mirror
    └── src/
        ├── main.rs
        ├── lib.rs
        ├── routes.rs             # one route per feature group
        └── pages/                # one module per demo page
```

Served by `viewer-ctl start demo-viewer` (default port: **3099**, picked to
avoid colliding with existing viewers).

## Feature inventory

The demo-viewer references the following feature specs. Each row is a section
of the SPA (left nav entry) and a module under `frontend/dioxus/src/pages/`.

### Backend infrastructure

| Page slug | Spec slug | Module |
|---|---|---|
| `server-infra` | `viewer-api/server-infra` | `pages/server_infra.rs` |
| `auth-middleware` | `viewer-api/auth-middleware` | `pages/auth_middleware.rs` |
| `pagination-query` | `viewer-api/pagination-query` | `pages/pagination_query.rs` |
| `sse` | `viewer-api/sse` | `pages/sse.rs` |
| `session` | `viewer-api/session` | `pages/session.rs` |
| `source` | `viewer-api/source` | `pages/source.rs` |
| `client-log` | `viewer-api/client-log` | `pages/client_log.rs` |
| `dev-proxy` | `viewer-api/dev-proxy` | `pages/dev_proxy.rs` |

### Frontend components

| Page slug | Spec slug | Module |
|---|---|---|
| `layout` | `viewer-api/components/layout` | `pages/layout.rs` |
| `tree-view` | `viewer-api/components/tree-view` | `pages/tree_view.rs` |
| `tab-bar` | `viewer-api/components/tab-bar` | `pages/tab_bar.rs` |
| `code-viewer` | `viewer-api/components/code-viewer` | `pages/code_viewer.rs` |
| `icons-spinner` | `viewer-api/components/icons-spinner` | `pages/icons_spinner.rs` |
| `theme-settings` | `viewer-api/theme-settings` | `pages/theme_settings.rs` |

### Visual / GPU / 3D

| Page slug | Spec slug | Module |
|---|---|---|
| `wgpu-overlay` | `viewer-api/effects/wgpu-overlay` | `pages/wgpu_overlay.rs` |
| `graph3d` | `viewer-api/components/graph3d` | `pages/graph3d.rs` |

### Cross-cutting

| Page slug | Spec slug | Module |
|---|---|---|
| `tracing` | `viewer-api/tracing` | `pages/tracing.rs` |
| `store-primitives` | `viewer-api/store-primitives` | `pages/store_primitives.rs` |

## Backend API surface

The demo-viewer's HTTP backend exposes a minimal router used by the demo
pages. All routes are namespaced under `/api/demo/`:

| Method+Path | Purpose | Spec |
|---|---|---|
| `GET /api/demo/health` | liveness | `viewer-api/server-infra` |
| `GET /api/demo/secured` | requires auth header | `viewer-api/auth-middleware` |
| `GET /api/demo/error/:kind` | returns each error variant | `viewer-api/auth-middleware` |
| `GET /api/demo/items?cursor&limit` | paginated list | `viewer-api/pagination-query` |
| `GET /api/demo/query?q=…&filter=…` | typed query parsing | `viewer-api/pagination-query` |
| `GET /api/demo/sse/stream` | server-sent events feed (~1/sec) | `viewer-api/sse` |
| `GET /api/demo/session` | session round-trip | `viewer-api/session` |
| `GET /api/demo/source/:path` | safe source-file serving | `viewer-api/source` |
| `POST /api/log/client` | reuses `client_log_router` | `viewer-api/client-log` |
| `GET /api/demo/graph` | static sample graph for graph3d | `viewer-api/components/graph3d` |

## Default state and seed data

- The demo-viewer ships with a tiny in-memory dataset for the paginated/query
  endpoints (50 deterministic items).
- The graph3d page renders a hard-coded 12-node / 18-edge sample graph.
- `auth-middleware` page accepts the bearer token `demo-token` (configurable
  via `--demo-token`).

## Cross-references

- Reference implementations are pulled from the existing viewers
  (`log-viewer`, `ticket-viewer`, `spec-viewer`) so the demo-viewer never
  contains "novel" usage that downstream viewers wouldn't recognise.
- Every demo page **must** include a small "Spec" link in its header that
  navigates to the corresponding spec-viewer entry (when running locally).

## Test strategy

- Each feature page has at least one e2e test under
  `tools/viewer/e2e/tests/demo-viewer/<page-slug>.spec.ts`.
- Tests run against the locally started demo-viewer
  (`viewer-ctl start demo-viewer`).
- Tests register themselves as `code_refs` on the corresponding feature spec
  (path + line range) so spec-viewer surfaces "verified by" links.
- The graph3d page reuses the WebGPU launch flags pattern documented in
  `tools/viewer/e2e/tests/dioxus/graph3d-right-drag.spec.ts`.

## Lifecycle

This spec moves to `reviewed` when the implementation tickets are accepted by
the user, `approved` when the demo-viewer scaffold is merged, `implemented`
when every feature page exists and its e2e test passes, and `verified` when
the manual-validation epic (see implementation tickets) is signed off.
