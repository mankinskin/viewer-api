# T7: Port — doc-viewer Leptos Frontend

## Problem

The doc-viewer currently uses a Preact/TS frontend with marked + highlight.js for markdown rendering and a tree-based crate browser. This needs a Leptos/WASM port that renders markdown as native DOM nodes (not innerHTML), uses the shared viewer-api-leptos crate (T6), and connects to the existing doc-viewer backend API.

## Reference: TS Implementation

### Backend API (tools/viewer/doc-viewer/src/http.rs)
- **L113–140**: `create_router()` — API routes mounted under `/api`
- **L143–180**: Route table:
  - `GET /api/docs` — list agent docs with optional type filter
  - `GET /api/docs/{filename}` — read single doc (Outline or Full detail)
  - `GET /api/docs/{filename}/ast` — markdown AST tree
  - `GET /api/crates` — list all indexed crates
  - `GET /api/crates/{name}` — crate summary with module tree + source links
  - `GET /api/crates/{name}/doc` — module-level documentation
  - `GET /api/source/{*path}` — source file content
  - `POST /api/query` — JQ query over docs
  - `GET /api/session` — read server session state
  - `POST /api/session` — write server session state
- **L72–110**: Response types: DocListResponse, DocContentResponse, CrateListResponse, CrateTreeResponse, CrateDocResponse, JqQueryResponse

### App.tsx (doc-viewer frontend)
- **L52–75**: Tri-pane layout: Header → FilterPanel → Sidebar (crate/doc tree) + main (DocViewer or FileViewer)

### Sidebar.tsx — Crate tree navigation
- **L8–18**: `useEffect()` — preloads unloaded crate module trees on mount
- **L20–75**: `convertNode()` — transforms TreeNode → SharedTreeNode with icons/tooltips
- **L77–145**: `handleSelect()` — actions: selectDoc, openSourceFile, openCategoryPage, loadCrateModules, openCrateDoc, toggleNodeExpanded  
- **L147–250**: Icon components: FolderIcon, CrateIcon, ModuleIcon, FileIcon, SourceFileIcon

### FileViewer.tsx — Markdown + code rendering
- **L33–35**: `isMarkdownFile()` — extension check (.md, .markdown)
- **L43–68**: Markdown: `marked.parse()` → `dangerouslySetInnerHTML` + highlighted code blocks via hljs
- **L70–85**: Non-markdown: falls back to CodeViewer component

### Components — full UI layer
- Breadcrumbs.tsx, CategoryPage.tsx, DocumentTabs.tsx, DocViewer.tsx, FileViewer.tsx, FilterPanel.tsx, Header.tsx, Sidebar.tsx

### Dependencies (package.json)
- @context-engine/viewer-api-frontend (shared), @preact/signals, highlight.js, marked, marked-highlight, preact, prismjs

## Design

### Step 1: Crate setup

```
tools/viewer/doc-viewer/frontend-leptos/
├── Cargo.toml
├── Trunk.toml
├── index.html
├── src/
│   ├── lib.rs
│   ├── app.rs
│   ├── api.rs          # Fetch wrappers for /api/* endpoints
│   ├── store.rs        # Signals: docs, crates, active_doc, breadcrumbs, session
│   ├── types.rs        # DocSummary, CrateSummary, CrateTree, DocContent etc.
│   ├── actions.rs      # load_docs, load_crates, select_doc, open_crate_doc
│   ├── markdown.rs     # pulldown-cmark → Leptos DOM nodes
│   └── components/
│       ├── mod.rs
│       ├── doc_viewer.rs      # Markdown document renderer
│       ├── file_viewer.rs     # Markdown or CodeViewer switch
│       ├── sidebar.rs         # Crate tree + doc tree
│       ├── breadcrumbs.rs     # Navigation breadcrumbs
│       ├── category_page.rs   # Category index pages
│       ├── document_tabs.rs   # Tabbed doc navigation
│       └── filter_panel.rs    # Type/tag filters
├── style.css
└── static/
```

```toml
[package]
name = "doc-viewer-leptos"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
viewer-api-leptos = { path = "../../viewer-api/frontend-leptos", features = ["syntect"] }
leptos = { version = "0.8", features = ["csr"] }
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["Window", "Document", "HtmlElement"] }
gloo-net = "0.7"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
pulldown-cmark = "0.12"
console_log = "1"
log = "0.4"
```

### Step 2: pulldown-cmark → Leptos DOM nodes

Instead of innerHTML (XSS risk with marked.parse()), parse markdown into a structured event stream and build Leptos views:

```rust
// src/markdown.rs
use pulldown_cmark::{Parser, Event, Tag, Options, CodeBlockKind};
use leptos::prelude::*;

pub fn render_markdown(content: &str) -> impl IntoView {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);
    
    let parser = Parser::new_ext(content, opts);
    let events: Vec<Event<'_>> = parser.collect();
    
    events_to_view(&events)
}

fn events_to_view(events: &[Event<'_>]) -> View {
    // Walk event stream, build nested view tree:
    // - Start(Heading) → <h1>..<h6> with id slug
    // - Start(Paragraph) → <p>
    // - Start(CodeBlock(Fenced(lang))) → <pre><code> with syntect highlighting
    // - Start(Table) → <table> with <thead>/<tbody>
    // - Start(List) → <ul>/<ol>
    // - Start(Link) → <a href=...>
    // - Start(Image) → <img src=...>
    // - Text(t) → text node
    // - Code(c) → <code class="inline">
    // - SoftBreak → " "
    // - HardBreak → <br/>
    // - TaskListMarker(checked) → <input type="checkbox" disabled checked=...>
    // Build recursively by consuming start/end pairs
}

fn highlight_code_block(code: &str, language: &str) -> impl IntoView {
    // Use viewer-api-leptos syntect feature
    #[cfg(feature = "syntect")]
    {
        let highlighted = viewer_api_leptos::syntax::highlight_code(code, language);
        view! {
            <pre class="code-block">
                <code>
                    <For each=move || highlighted.clone().into_iter().enumerate()
                         key=|(i, _)| *i
                         let:(i, line_html)>
                        <span class="code-line" inner_html=line_html />
                    </For>
                </code>
            </pre>
        }
    }
    #[cfg(not(feature = "syntect"))]
    {
        view! { <pre class="code-block"><code>{code}</code></pre> }
    }
}
```

### Step 3: Store — global signals

```rust
// src/store.rs
pub struct DocStore {
    // Doc list
    pub docs: RwSignal<Vec<DocSummary>>,
    pub doc_filter_type: RwSignal<Option<String>>,
    pub doc_search: RwSignal<String>,
    
    // Crate list + trees
    pub crates: RwSignal<Vec<CrateSummary>>,
    pub crate_trees: RwSignal<HashMap<String, CrateTree>>,
    
    // Active content
    pub active_doc: RwSignal<Option<DocContent>>,
    pub active_source_file: RwSignal<Option<SourceFile>>,
    pub active_crate_doc: RwSignal<Option<CrateDocContent>>,
    
    // Navigation
    pub breadcrumbs: RwSignal<Vec<Breadcrumb>>,
    pub active_view: RwSignal<DocView>,  // enum { DocList, DocDetail, CrateDetail, SourceFile, CategoryPage }
    
    // Session persistence
    pub session: RwSignal<Option<SessionState>>,
    
    // Theme (from viewer-api-leptos)
    pub theme_store: ThemeStore,
    
    // Code viewer panel
    pub code_viewer_file: RwSignal<Option<String>>,
    pub code_viewer_content: RwSignal<String>,
    pub code_viewer_line: RwSignal<Option<usize>>,
    
    // Loading states
    pub is_loading: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
}
```

### Step 4: Sidebar — crate module tree

Port the TS Sidebar's `convertNode()` + `handleSelect()` pattern using TreeView from viewer-api-leptos:

```rust
// src/components/sidebar.rs
use viewer_api_leptos::components::{TreeView, TreeNode, TreeNodeIcon};

#[component]
pub fn DocSidebar(store: DocStore) -> impl IntoView {
    // Preload crate trees on mount
    create_effect(move |_| {
        for crate_summary in store.crates.get().iter() {
            if !store.crate_trees.get().contains_key(&crate_summary.name) {
                spawn_local(load_crate_modules(&store, &crate_summary.name));
            }
        }
    });
    
    // Build tree from docs + crates
    let tree_nodes = create_memo(move |_| {
        let mut nodes = vec![];
        
        // Agent docs section
        let doc_children: Vec<TreeNode> = store.docs.get().iter()
            .map(|d| TreeNode {
                id: d.filename.clone(),
                label: d.title.clone().unwrap_or(d.filename.clone()),
                icon: TreeNodeIcon::File,
                children: vec![],
                is_expanded: false,
            })
            .collect();
        nodes.push(TreeNode {
            id: "agent-docs".into(),
            label: "Agent Docs".into(),
            icon: TreeNodeIcon::Folder,
            children: doc_children,
            is_expanded: true,
        });
        
        // Crate docs section — each crate expands to module tree
        for crate_summary in store.crates.get().iter() {
            let modules = store.crate_trees.get()
                .get(&crate_summary.name)
                .map(|tree| convert_crate_tree(tree))
                .unwrap_or_default();
            nodes.push(TreeNode {
                id: format!("crate:{}", crate_summary.name),
                label: crate_summary.name.clone(),
                icon: TreeNodeIcon::Crate,
                children: modules,
                is_expanded: false,
            });
        }
        
        nodes
    });
    
    let on_select = Callback::new(move |node_id: String| {
        // Route to appropriate action based on node ID prefix
        if node_id.starts_with("crate:") { ... }
        else if node_id.starts_with("module:") { ... }
        else if node_id.starts_with("source:") { ... }
        else { select_doc(&store, &node_id); }
    });
    
    view! { <TreeView nodes=tree_nodes on_select=on_select indent_px=8 /> }
}
```

### Step 5: DocViewer — markdown rendering

```rust
// src/components/doc_viewer.rs
#[component]
pub fn DocViewer(store: DocStore) -> impl IntoView {
    let content_view = create_memo(move |_| {
        store.active_doc.get().map(|doc| {
            render_markdown(&doc.content)
        })
    });
    
    view! {
        <div class="doc-viewer">
            <Show when=move || store.active_doc.get().is_some()
                  fallback=|| view! { <div class="placeholder">"Select a document"</div> }>
                <div class="doc-content markdown-body">
                    {content_view}
                </div>
            </Show>
        </div>
    }
}
```

### Step 6: FileViewer — markdown vs code

```rust
// src/components/file_viewer.rs
use viewer_api_leptos::components::FileContentViewer;

#[component]
pub fn FileViewer(store: DocStore) -> impl IntoView {
    let is_markdown = move || {
        store.code_viewer_file.get()
            .map(|f| f.ends_with(".md") || f.ends_with(".markdown"))
            .unwrap_or(false)
    };
    
    view! {
        <Show when=is_markdown
              fallback=move || view! {
                  <FileContentViewer
                      file=store.code_viewer_file
                      content=store.code_viewer_content
                      highlight_line=Some(store.code_viewer_line)
                  />
              }>
            {move || {
                let content = store.code_viewer_content.get();
                view! { <div class="markdown-body">{render_markdown(&content)}</div> }
            }}
        </Show>
    }
}
```

### Step 7: Breadcrumbs

```rust
// src/components/breadcrumbs.rs
#[derive(Clone)]
pub struct Breadcrumb {
    pub label: String,
    pub action: Option<Callback<()>>,  // None = current (no click)
}

#[component]
pub fn Breadcrumbs(crumbs: Signal<Vec<Breadcrumb>>) -> impl IntoView {
    view! {
        <nav class="breadcrumbs">
            <For each=move || crumbs.get().into_iter().enumerate()
                 key=|(i, _)| *i
                 let:(i, crumb)>
                {if i > 0 { view! { <span class="separator">"›"</span> }.into_any() } else { ().into_any() }}
                {match crumb.action {
                    Some(action) => view! {
                        <button class="crumb" on:click=move |_| action.call(())>
                            {&crumb.label}
                        </button>
                    }.into_any(),
                    None => view! { <span class="crumb current">{&crumb.label}</span> }.into_any(),
                }}
            </For>
        </nav>
    }
}
```

### Step 8: App root — tri-pane layout

```rust
// src/app.rs
use viewer_api_leptos::components::{TriPaneLayout, Header, ResizeHandle};

#[component]
pub fn App() -> impl IntoView {
    let store = DocStore::new();
    provide_context(store.clone());
    
    // Load initial data
    spawn_local(async move {
        load_docs(&store).await;
        load_crates(&store).await;
        restore_session(&store).await;
    });
    
    let sidebar_width = create_rw_signal(280.0);
    
    view! {
        <TriPaneLayout
            header=ViewFn::from(move || view! {
                <Header title="Doc Viewer".to_string()>
                    <ThemeButton />
                    <FilterToggle />
                </Header>
            })
            sidebar=ViewFn::from(move || view! {
                <DocSidebar store=store.clone() />
            })
            main_content=ViewFn::from(move || view! {
                <Breadcrumbs crumbs=store.breadcrumbs.into() />
                <div class="doc-main">
                    {move || match store.active_view.get() {
                        DocView::DocDetail => view! { <DocViewer store=store.clone() /> }.into_any(),
                        DocView::SourceFile => view! { <FileViewer store=store.clone() /> }.into_any(),
                        DocView::CategoryPage => view! { <CategoryPage store=store.clone() /> }.into_any(),
                        _ => view! { <DocList store=store.clone() /> }.into_any(),
                    }}
                </div>
            })
            sidebar_width=sidebar_width
        />
    }
}
```

### Step 9: Server-side session persistence

```rust
// src/actions.rs
pub async fn restore_session(store: &DocStore) {
    match gloo_net::http::Request::get("/api/session").send().await {
        Ok(resp) if resp.ok() => {
            if let Ok(session) = resp.json::<SessionState>().await {
                // Restore active doc, expanded nodes, scroll position
                store.session.set(Some(session));
            }
        }
        _ => {}
    }
}

pub async fn save_session(store: &DocStore) {
    let state = SessionState {
        active_doc: store.active_doc.get().map(|d| d.filename.clone()),
        expanded_nodes: /* collect from tree */,
        scroll_position: /* read from DOM */,
    };
    let _ = gloo_net::http::Request::post("/api/session")
        .json(&state).unwrap()
        .send().await;
}
```

### Step 10: Backend integration

The existing doc-viewer backend (`tools/viewer/doc-viewer/src/http.rs`) already serves all the API endpoints. The Leptos frontend just needs to be served as static files. Add a trunk build step and configure the backend to serve `dist/` at `/`:

```rust
// In doc-viewer main.rs — add static file serving for Leptos build
let app = Router::new()
    .nest("/api", api_router)
    .fallback_service(ServeDir::new(&static_dir));
```

## Files to Create

| File | Purpose |
|------|---------|
| `frontend-leptos/Cargo.toml` | Crate manifest |
| `frontend-leptos/Trunk.toml` | Trunk build config |
| `frontend-leptos/index.html` | WASM entry point |
| `frontend-leptos/src/lib.rs` | Module declarations |
| `frontend-leptos/src/app.rs` | Root App with tri-pane layout |
| `frontend-leptos/src/api.rs` | Fetch wrappers for 9 API endpoints |
| `frontend-leptos/src/store.rs` | DocStore with all signals |
| `frontend-leptos/src/types.rs` | API response types |
| `frontend-leptos/src/actions.rs` | Async actions (load, select, session) |
| `frontend-leptos/src/markdown.rs` | pulldown-cmark → Leptos DOM |
| `frontend-leptos/src/components/*.rs` | All 7 components |
| `frontend-leptos/style.css` | Full CSS |

## Files to Modify

| File | Change |
|------|--------|
| `tools/viewer/doc-viewer/Cargo.toml` | Add optional `leptos-frontend` feature |
| `tools/viewer/doc-viewer/src/main.rs` or `http_server.rs` | Serve Leptos dist as static fallback |
| Workspace `Cargo.toml` | Add doc-viewer-leptos to members |

## Acceptance Criteria

1. pulldown-cmark parses markdown to Leptos DOM nodes (no innerHTML / dangerouslySetInnerHTML)
2. Code blocks highlighted via syntect (viewer-api-leptos `syntect` feature)
3. GFM tables render as `<table>` with proper headers
4. Task lists render as checkbox items
5. Full crate module tree in sidebar (TreeView from viewer-api-leptos)
6. Sidebar preloads module trees for all indexed crates
7. Breadcrumb navigation between docs, crates, modules, source files
8. Source file viewing via CodeViewer (viewer-api-leptos)
9. Session persistence: GET/POST /api/session restores last-viewed doc on refresh
10. Theme system via shared ThemeStore (viewer-api-leptos)
11. All 9 backend API endpoints consumed correctly
12. trunk build produces working WASM bundle served by existing backend
