Add foundational visual primitives to `viewer-api-dioxus` that doc-viewer relies on but spec-viewer currently lacks.

## Deliverables

1. `components/breadcrumbs.rs`
   - `Breadcrumbs { items: Vec<BreadcrumbItem> }` component
   - `BreadcrumbItem { label, icon: Option<Element>, href: Option<String>, on_click: Option<EventHandler<()>> }`
   - Reuse icons from `icons.rs`
   - CSS: `viewer-api/public/css/breadcrumbs.css` (`.breadcrumbs`, `.breadcrumbs__item`, `.breadcrumbs__sep`, `.breadcrumbs__item--current`)

2. `components/modal.rs`
   - `Overlay { open: bool, on_close: EventHandler<()>, children: Element }`
   - Backdrop click + Esc key dismiss (use `gloo-events`)
   - Body-scroll lock while open
   - CSS: `modal.css` (`.modal-backdrop`, `.modal-panel`)
   - Pointer-events: only `.modal-panel` captures clicks; backdrop captures only when `open`

3. `components/meta_header.rs`
   - `MetaHeader { title, date: Option<String>, tags: Vec<String>, status: Option<String> }`
   - `Chip { text, kind: ChipKind }` and `ChipRow { children }`
   - CSS: `chip.css`, `meta-header.css` using `--accent-*` tokens

4. `components/cards.rs`
   - `Card { icon: Option<Element>, title: String, description: Option<String>, badge: Option<String>, on_click: Option<EventHandler<()>> }`
   - `CardGrid { children }` (responsive auto-fit)
   - `CardSection { title, count: Option<usize>, children }`
   - CSS: `cards.css`

## Wiring

- All four CSS files imported from `viewer-api.css`
- New components re-exported from `components/mod.rs` and `lib.rs`

## Acceptance criteria

- `cargo check -p viewer-api-dioxus --target wasm32-unknown-unknown` passes
- A demo route in spec-viewer (or a temporary `/__demo` page) renders one of each component to verify visually
- All hardcoded colours use CSS tokens, not literals
- BEM naming convention followed
