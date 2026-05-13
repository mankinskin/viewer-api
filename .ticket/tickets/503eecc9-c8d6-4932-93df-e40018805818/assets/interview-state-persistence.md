# Interview: State Persistence and Runtime Architecture

**Date:** 2026-04-08
**Applies to:** `c2f04936` (State persistence), `503eecc9` (URL/session), Epic `35a6d14b`

## Question

Current ticket-viewer stores per-workspace state in localStorage. Should the Dioxus port also use localStorage, or prefer server-side sessions? Should URL routing include deep-links to specific tickets?

## Answer

**We want to support a localStorage user session. The viewer is designed to work locally, even distributing the server-side logic to the WASM package and performing all ticket storage on the client side. We want to maintain close distance to the native Rust implementation using a minimal browser runtime for window management and rendering, but being both a native Rust executable and a minimal WASM view layer, talking through the same native memory (future vision).**

## Implications

- **localStorage for session state** (selected ticket, sidebar width, theme, collapse state)
- **Future architecture vision:** the WASM package should embed ticket-api logic directly
  - Client-side ticket storage (IndexedDB or in-memory) for offline/standalone mode
  - Same Rust code runs natively (server) or in WASM (client) — shared via feature flags
  - Browser provides only window management and rendering primitives
- **Short-term:** Use ticket-http as the backend, but design the API client layer as a trait so it can be swapped for an embedded implementation later
- **Abstraction layer:** Define a `TicketBackend` trait that both HTTP-client and embedded-store implement
- The WASM build should eventually be a standalone app that doesn't require a server
- Deep-link URL routing: yes, support `#/workspace/{name}/ticket/{id}` style routes
- This fundamentally shapes crate boundaries — the ticket-api crate must compile to wasm32-unknown-unknown
