# viewer-api: store primitives

Canonical specification for the shared client-side store helpers under
`viewer-api/frontend/dioxus/src/store/` — the localStorage-backed signal
helpers reused by every viewer (theme, layout, settings, last-opened ids).

## Public surface

- `store::persistent_signal<T>(key, default) -> Signal<T>` —
  `serde_json` round-trip on read, debounced write on update.
- `store::clear(key)` and `store::clear_all(prefix)`.
- `store::PersistentScope` — namespaced key derivation
  (`viewer-name:scope:key`).

## Demo behavior

The `pages/store_primitives.rs` page demonstrates:

1. Three persistent signals: a counter, a free-text input, and a JSON
   structure (object with two fields).
2. A live readout of the underlying localStorage values
   (`viewer-api-demo:counter`, `…:text`, `…:json`).
3. Buttons: "Increment", "Reset", "Clear localStorage".
4. A second tab/window scenario: the page reflects external localStorage
   updates within 1 s (or on focus).

## Acceptance behavior (validated by e2e)

- Setting the counter to 7 and reloading the page restores the value.
- Clearing localStorage resets all three signals to their defaults.
- Writing to the localStorage key from a second `BrowserContext` updates
  the first context's UI on focus / next tick.

## Code references

- `tools/viewer/viewer-api/frontend/dioxus/src/store/`
- `tools/viewer/e2e/tests/demo-viewer/store-primitives.spec.ts`
