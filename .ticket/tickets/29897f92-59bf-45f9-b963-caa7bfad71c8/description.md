# T4: Feature — UI Polish (Tab Bar, Sidebar, Resizable Panels)

## Problem

The Leptos frontend has a minimal tab bar (20px, uppercase, no icons) and a flat sidebar (220px, no tree indentation, no resize). The TS version has a polished tab bar (32px, icons, active accents), a full TreeView with indentation and expand/collapse, and a ResizeHandle component for draggable panel resizing.

## Current State (Leptos)

### app.rs L7–42
- Tab bar rendered as `<TabBar />`
- Sidebar rendered as `<Sidebar />`
- Layout: `lv-app > Header + lv-body(Sidebar + lv-main(TabBar + view-container))`

### style.css
- Tab bar: ~20px height, uppercase labels, simple background toggle for active
- Sidebar: 220px hardcoded width, flat file list

### Missing:
- No `ResizeHandle` component
- No `TreeView` with indentation
- No expand/collapse chevrons
- No colored icons (folder/file)
- No sidebar collapse toggle
- No tab icons

## Reference: TS Implementation

### TabBar.tsx (viewer-api-frontend)
- **L1–6**: `Tab` interface: `id, label, icon?, closeable?, modified?`
- **L8–19**: `TabBarProps`: `tabs, activeTabId, onSelect, onClose?, rightContent?`
- **L21–55**: Renders tab buttons with icon, label, modified indicator (orange •), close button (×)
- Active tab: `class="tab active"` with accent bottom border
- 32px height, 180px max-width per tab, horizontal scroll with hidden scrollbar

### ResizeHandle.tsx (viewer-api-frontend)
- **L1–9**: Props: `onResize(delta)`, `onResizeStart?`, `onResizeEnd?`, `direction: 'horizontal'|'vertical'`, `edge?`, `deltaSign?: 1|-1`
- **L27–32**: mousedown: set isDragging, capture lastPos, set `cursor: col-resize` on body, `userSelect: none`
- **L34–48**: mousemove: calculate delta, accumulate in `pendingDelta`, batch via `requestAnimationFrame` — only 1 rAF at a time
- **L50–63**: mouseup: clear dragging, restore cursor/userSelect, cancel pending rAF, flush accumulated delta

### TreeView.tsx (viewer-api-frontend)
- **L1–11**: `TreeNode<T>`: `id, label, icon?, children?, data?, tooltip?, badge?`
- **L13–23**: `TreeViewProps<T>`: `nodes, selectedId?, onSelect?, defaultExpanded?, expanded?, onToggle?`
- **L49–150**: `TreeItem` recursive component
- **L98**: Indentation: `paddingLeft: ${depth * 8}px`
- **L116–119**: Chevron: `class="tree-toggle expanded"` rotates 90°
- **L81–91**: Icon: defaults to FolderIcon (has children) or FileIcon

## Design

### Step 1: ResizeHandle component

Port the TS ResizeHandle to Leptos. This is the foundation — used by sidebar and later by code viewer panel.

```rust
// src/components/resize_handle.rs
#[component]
pub fn ResizeHandle(
    /// Called with pixel delta during drag
    on_resize: Callback<f64>,
    /// Drag direction
    #[prop(default = "horizontal")]
    direction: &'static str,
    /// Multiplier for delta (1 or -1, for left/right anchored panes)
    #[prop(default = 1.0)]
    delta_sign: f64,
) -> impl IntoView { ... }
```

**Event handling pattern** (Leptos equivalent of TS):
1. `on:mousedown` on the handle element → set dragging flag, capture `client_x`/`client_y`
2. Register `mousemove` + `mouseup` listeners on `document` (via `web_sys::EventTarget::add_event_listener`)
3. mousemove: compute delta, accumulate in `pending_delta: Rc<Cell<f64>>`
4. Use `request_animation_frame` to batch — only schedule one rAF at a time (track with `raf_id: Rc<Cell<Option<i32>>>`)
5. In rAF callback: read + reset `pending_delta`, call `on_resize.call(delta * delta_sign)`
6. mouseup: remove document listeners, restore cursor, cancel pending rAF, flush remaining delta

**Cursor management**:
```rust
// During drag:
document.body().unwrap().style().set_property("cursor", "col-resize").ok();
document.body().unwrap().style().set_property("user-select", "none").ok();

// On release:
document.body().unwrap().style().remove_property("cursor").ok();
document.body().unwrap().style().remove_property("user-select").ok();
```

**CSS**:
```css
.resize-handle {
    flex-shrink: 0;
    background: transparent;
    transition: background 0.15s;
}
.resize-handle-horizontal {
    width: 4px;
    cursor: col-resize;
}
.resize-handle-vertical {
    height: 4px;
    cursor: row-resize;
}
.resize-handle:hover,
.resize-handle.dragging {
    background: var(--accent-blue);
}
```

### Step 2: TreeView component

Port TreeView for use in sidebar. This component goes into `viewer-api-leptos` shared crate (T6), but for now build it in log-viewer.

```rust
// src/components/tree_view.rs

#[derive(Clone)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub icon: TreeIcon,         // Folder, File, Doc, Custom(View)
    pub children: Vec<TreeNode>,
    pub badge: Option<String>,
}

#[component]
pub fn TreeView(
    nodes: Signal<Vec<TreeNode>>,
    selected_id: Signal<Option<String>>,
    on_select: Callback<String>,
    #[prop(default = vec![])]
    default_expanded: Vec<String>,
) -> impl IntoView { ... }
```

**TreeItem** recursive component:
```rust
#[component]
fn TreeItem(
    node: TreeNode,
    depth: usize,
    selected_id: Signal<Option<String>>,
    expanded: RwSignal<HashSet<String>>,
    on_select: Callback<String>,
) -> impl IntoView {
    let has_children = !node.children.is_empty();
    let is_expanded = move || expanded.get().contains(&node.id);
    let is_selected = move || selected_id.get().as_deref() == Some(&node.id);

    view! {
        <div
            class="tree-item-row"
            class:selected=is_selected
            style=format!("padding-left: {}px", depth * 8)
            on:click=move |_| on_select.call(node.id.clone())
        >
            <span class="tree-toggle"
                  class:expanded=is_expanded
                  class:empty=(!has_children)
                  on:click=move |e| { e.stop_propagation(); toggle_expand(); }
            >
                <ChevronIcon />
            </span>
            <span class="tree-icon">{render_icon(&node.icon, has_children)}</span>
            <span class="tree-label">{&node.label}</span>
            {node.badge.map(|b| view! { <span class="tree-badge">{b}</span> })}
        </div>
        <Show when=move || is_expanded() && has_children>
            <For each=move || node.children.clone()
                 key=|n| n.id.clone()
                 let:child>
                <TreeItem node=child depth=depth+1 /* ... */ />
            </For>
        </Show>
    }
}
```

**CSS**:
```css
.tree-item-row {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    cursor: pointer;
    font-size: 13px;
    white-space: nowrap;
}
.tree-item-row:hover { background: var(--bg-hover); }
.tree-item-row.selected { background: var(--bg-active); }
.tree-toggle { width: 16px; text-align: center; transition: transform 0.15s; }
.tree-toggle.expanded { transform: rotate(90deg); }
.tree-toggle.empty { visibility: hidden; }
.tree-icon { width: 16px; height: 16px; flex-shrink: 0; }
.tree-badge { margin-left: auto; font-size: 11px; color: var(--text-muted); }
```

### Step 3: Update tab bar

**app.rs / tab_bar.rs** — Update the tab bar component:
- Height: 32px
- Each tab: icon + label (no close buttons — fixed tabs)
- Active tab: bottom 2px accent border
- Horizontal scroll with `overflow-x: auto; scrollbar-width: none`

```rust
#[component]
pub fn TabBar(
    active_tab: Signal<ViewTab>,
    on_select: Callback<ViewTab>,
) -> impl IntoView {
    let tabs = vec![
        (ViewTab::Logs, "Logs", LogsIcon),
        (ViewTab::Hypergraph, "Hypergraph", GraphIcon),
        (ViewTab::Settings, "Settings", SettingsIcon),
    ];
    view! {
        <div class="tab-bar">
            <div class="tabs">
                {tabs.into_iter().map(|(tab, label, icon)| {
                    view! {
                        <button
                            class="tab"
                            class:active=move || active_tab.get() == tab
                            on:click=move |_| on_select.call(tab)
                        >
                            <span class="tab-icon">{icon()}</span>
                            <span class="tab-label">{label}</span>
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
```

**CSS**:
```css
.tab-bar { height: 32px; display: flex; border-bottom: 1px solid var(--border-color); }
.tabs { display: flex; flex: 1; overflow-x: auto; scrollbar-width: none; }
.tabs::-webkit-scrollbar { display: none; }
.tab {
    display: flex; align-items: center; gap: 6px;
    padding: 0 12px; font-size: 13px;
    border: none; background: transparent; color: var(--text-secondary);
    cursor: pointer; white-space: nowrap; border-bottom: 2px solid transparent;
}
.tab:hover { color: var(--text-primary); background: var(--bg-hover); }
.tab.active { color: var(--text-primary); border-bottom-color: var(--accent-orange); }
.tab-icon { width: 14px; height: 14px; }
```

### Step 4: Sidebar with resize and collapse

Wire the sidebar width to a signal + ResizeHandle:

```rust
// app.rs
let sidebar_width = create_rw_signal(260.0_f64);
let sidebar_collapsed = create_rw_signal(false);

view! {
    <div class="lv-body">
        <Show when=move || !sidebar_collapsed.get()>
            <aside class="lv-sidebar" style=move || format!("width: {}px", sidebar_width.get())>
                <div class="sidebar-header">
                    <span class="sidebar-title">"Files"</span>
                    <span class="sidebar-badge">{file_count}</span>
                    <button class="sidebar-collapse" on:click=move |_| sidebar_collapsed.set(true)>
                        <ChevronLeftIcon />
                    </button>
                </div>
                <TreeView nodes=file_tree selected_id=selected_file on_select=on_file_select />
            </aside>
            <ResizeHandle
                on_resize=Callback::new(move |delta| {
                    let new_w = (sidebar_width.get() + delta).clamp(160.0, 500.0);
                    sidebar_width.set(new_w);
                })
                direction="horizontal"
            />
        </Show>
        <Show when=move || sidebar_collapsed.get()>
            <button class="sidebar-expand" on:click=move |_| sidebar_collapsed.set(false)>
                <ChevronRightIcon />
            </button>
        </Show>
        <main class="lv-main"> ... </main>
    </div>
}
```

### Step 5: SVG icons

Create a minimal icon module with inline SVG for: Logs, Graph, Settings, ChevronRight, ChevronLeft, Folder, File, Document, Palette.

```rust
// src/components/icons.rs
#[component]
pub fn ChevronRightIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 16 16" width="16" height="16" fill="currentColor">
            <path d="M6 3l5 5-5 5z"/>
        </svg>
    }
}
// ... similar for other icons
```

## Files to Create

| File | Purpose |
|------|---------|
| `src/components/resize_handle.rs` | ResizeHandle with rAF-batched drag |
| `src/components/tree_view.rs` | TreeView + TreeItem recursive component |
| `src/components/icons.rs` | SVG icon components |

## Files to Modify

| File | Change |
|------|--------|
| `src/components/tab_bar.rs` | Rewrite: 32px, icons, active accent border |
| `src/app.rs` | Wire sidebar width signal + ResizeHandle + collapse toggle |
| `src/components/sidebar.rs` | Use TreeView instead of flat list, add header with badge + collapse |
| `src/components/mod.rs` | Register new modules |
| `style.css` | Tab bar (32px, icon+label, active border), tree view, resize handle, sidebar collapse |

## Acceptance Criteria

1. Tab bar: 32px height, icon + label per tab, active tab with accent bottom border, horizontal scroll
2. Sidebar: TreeView with 8px-per-level indentation, chevron expand/collapse, colored icons, header with badge
3. ResizeHandle: mousedown/mousemove/mouseup with rAF-batched delta, col-resize cursor, user-select none
4. Sidebar width driven by signal + ResizeHandle, default 260px, min 160px, max 500px
5. Sidebar collapse toggle button
6. TreeView component designed for extraction to viewer-api-leptos (T6)
7. No right panel or CodeViewer in this ticket
