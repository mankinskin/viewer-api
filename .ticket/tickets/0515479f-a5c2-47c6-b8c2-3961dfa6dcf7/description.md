---
tags: `#plan` `#context-trace` `#debugging` `#api`
summary: The existing MCP Docs Server (`tools/mcp-docs-server/`) manages **agent workflow documentation** in `agents/`:
status: 📋
---

# MCP Docs Server Update - Work Issues

**Date:** 2026-02-15  
**Status:** Planning  
**Purpose:** Extend the MCP docs server to support crate documentation in `crates/*/agents/docs/`

---

## Overview

### Current State

The existing MCP Docs Server (`tools/mcp-docs-server/`) manages **agent workflow documentation** in `agents/`:
- Document types: guide, plan, implemented, bug-report, analysis
- Format: Markdown with YAML frontmatter
- Features: create, list, update_meta, search_by_tag, validate, regenerate_index, browse, search_content

### New Requirement

Support **crate API documentation** in `crates/*/agents/docs/`:
- Structure: Mirror source code module hierarchy
- Format: YAML (`index.yaml`) + Markdown (`README.md`)
- Schema: `name`, `description`, `modules`, `exported_items`, `dependencies`, `features`, `files`, `key_types`

### Goal

Agents should easily:
1. **Browse** crate documentation during implementation
2. **Update** documentation after making code changes
3. **Search** across both agent workflow docs AND crate API docs

---

## Work Issues

### Issue 1: Define CrateDoc Schema

**Priority:** High  
**Type:** Schema Design

Add new schema types for crate documentation:

```rust
// New DocType variants or separate enum
pub enum CrateDocType {
    CrateIndex,      // crates/*/agents/docs/index.yaml
    ModuleIndex,     // crates/*/agents/docs/<module>/index.yaml
    ModuleReadme,    // crates/*/agents/docs/<module>/README.md
}

// Crate-level metadata (index.yaml)
pub struct CrateMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub modules: Vec<ModuleRef>,
    pub exported_items: ExportedItems,
    pub dependencies: Vec<String>,
    pub features: Vec<String>,
}

// Module-level metadata (subdir/index.yaml)
pub struct ModuleMetadata {
    pub name: String,
    pub description: String,
    pub submodules: Option<Vec<SubmoduleRef>>,
    pub files: Vec<FileEntry>,
    pub key_types: Option<Vec<TypeEntry>>,
}
```

**Files to modify:** `src/schema.rs`

---

### Issue 2: Add YAML Parser Support

**Priority:** High  
**Type:** Implementation

Current parser handles Markdown + YAML frontmatter. Need to add pure YAML parsing:

```rust
// New function in parser.rs
pub fn parse_yaml_index<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let content = fs::read_to_string(path).ok()?;
    serde_yaml::from_str(&content).ok()
}
```

**Files to modify:** `src/parser.rs`  
**Dependencies to add:** `serde_yaml`

---

### Issue 3: Implement Crate Discovery

**Priority:** High  
**Type:** Implementation

Scan `crates/` directory to find all context-* crates with documentation:

```rust
impl DocsManager {
    /// Discover all crates with agents/docs/ directories
    pub fn discover_crates(&self) -> Vec<CrateInfo> {
        let crates_dir = self.agents_dir.parent()?.join("crates");
        // Scan for crates/*/agents/docs/index.yaml
    }
    
    /// Get module hierarchy for a crate
    pub fn get_crate_modules(&self, crate_name: &str) -> ModuleTree {
        // Recursively scan module directories
    }
}
```

**Files to modify:** `src/tools.rs`

---

### Issue 4: Add `list_crates` Tool

**Priority:** High  
**Type:** New Tool

```rust
/// List all crates with documentation
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListCratesInput {
    /// Optional: filter to crates matching pattern (e.g., "context-*")
    #[serde(default)]
    pattern: Option<String>,
}

// Output: List of {name, version, description, module_count, has_readme}
```

**Files to modify:** `src/main.rs`, `src/tools.rs`

---

### Issue 5: Add `browse_crate` Tool

**Priority:** High  
**Type:** New Tool

```rust
/// Browse a crate's documentation structure
#[derive(Debug, Deserialize, JsonSchema)]
pub struct BrowseCrateInput {
    /// Crate name (e.g., "context-trace")
    crate_name: String,
    /// Optional: specific module path (e.g., "graph/vertex")
    #[serde(default)]
    module_path: Option<String>,
    /// Detail level: "overview" (modules only), "full" (with types)
    #[serde(default = "default_overview")]
    detail: String,
}

// Output: Hierarchical module tree with descriptions
```

**Files to modify:** `src/main.rs`, `src/tools.rs`

---

### Issue 6: Add `read_crate_doc` Tool

**Priority:** High  
**Type:** New Tool

```rust
/// Read crate or module documentation
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadCrateDocInput {
    /// Crate name
    crate_name: String,
    /// Module path (empty for crate root)
    #[serde(default)]
    module_path: Option<String>,
    /// What to read: "index" (yaml), "readme" (markdown), "both"
    #[serde(default = "default_both")]
    content: String,
}
```

**Files to modify:** `src/main.rs`, `src/tools.rs`

---

### Issue 7: Add `update_crate_doc` Tool

**Priority:** High  
**Type:** New Tool

Enable agents to update crate documentation after code changes:

```rust
/// Update crate or module documentation
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateCrateDocInput {
    /// Crate name
    crate_name: String,
    /// Module path (empty for crate root)
    #[serde(default)]
    module_path: Option<String>,
    /// Update index.yaml fields (optional)
    #[serde(default)]
    index_updates: Option<IndexUpdates>,
    /// New README content (optional, replaces entire file)
    #[serde(default)]
    readme_content: Option<String>,
}

pub struct IndexUpdates {
    pub description: Option<String>,
    pub files: Option<Vec<FileEntry>>,
    pub key_types: Option<Vec<TypeEntry>>,
    pub exported_items: Option<ExportedItems>,
}
```

**Files to modify:** `src/main.rs`, `src/tools.rs`

---

### Issue 8: Add `create_module_doc` Tool

**Priority:** Medium  
**Type:** New Tool

When a new module is added to source code, create its documentation:

```rust
/// Create documentation for a new module
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateModuleDocInput {
    /// Crate name
    crate_name: String,
    /// New module path (e.g., "graph/vertex/new_submod")
    module_path: String,
    /// Module description
    description: String,
    /// Optional initial files list
    #[serde(default)]
    files: Option<Vec<FileEntry>>,
    /// Optional initial key_types
    #[serde(default)]
    key_types: Option<Vec<String>>,
}
```

**Files to modify:** `src/main.rs`, `src/tools.rs`

---

### Issue 9: Add `search_crate_docs` Tool

**Priority:** Medium  
**Type:** New Tool

Search across crate documentation:

```rust
/// Search crate documentation
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchCrateDocsInput {
    /// Search query
    query: String,
    /// Search scope: "types", "modules", "files", "all"
    #[serde(default = "default_all")]
    scope: String,
    /// Optional: filter to specific crates
    #[serde(default)]
    crate_filter: Option<Vec<String>>,
}

// Output: Matches with crate, module path, and context
```

**Files to modify:** `src/main.rs`, `src/tools.rs`

---

### Issue 10: Add `validate_crate_docs` Tool

**Priority:** Medium  
**Type:** New Tool

Validate crate documentation:

```rust
/// Validate crate documentation structure
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateCrateDocsInput {
    /// Optional: specific crate (validates all if omitted)
    #[serde(default)]
    crate_name: Option<String>,
    /// Check for stale docs (modules in docs but not in src)
    #[serde(default)]
    check_stale: bool,
    /// Check for missing docs (modules in src but not in docs)
    #[serde(default)]
    check_missing: bool,
}

// Validations:
// - index.yaml parse errors
// - Missing README.md files
// - Stale module docs (module removed from src)
// - Missing module docs (module exists in src but not docs)
// - Broken submodule references
```

**Files to modify:** `src/main.rs`, `src/tools.rs`

---

### Issue 11: Add `sync_crate_docs` Tool

**Priority:** Low (nice-to-have)  
**Type:** New Tool

Auto-generate/update crate docs from source:

```rust
/// Sync documentation with source code
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SyncCrateDocsInput {
    /// Crate name
    crate_name: String,
    /// What to sync: "structure" (add missing modules), "exports" (scan lib.rs)
    mode: String,
    /// Dry run - show what would change without applying
    #[serde(default)]
    dry_run: bool,
}
```

This would scan source files to:
- Detect new modules and create stub documentation
- Parse `lib.rs` exports to update `exported_items`
- Mark stale modules

**Files to modify:** `src/main.rs`, `src/tools.rs`

---

### Issue 12: Update Agent Instructions

**Priority:** High  
**Type:** Documentation

Update `.agents/instructions.md` and `AGENTS.md` to:

1. Add crate documentation to priority order:
   ```
   ## Documentation Resources
   1. `CHEAT_SHEET.md` - Types, patterns, gotchas
   2. `agents/guides/INDEX.md` - How-to guides
   3. `crates/<crate>/agents/docs/` - API documentation  <-- NEW
   4. `crates/<crate>/HIGH_LEVEL_GUIDE.md` - Concepts
   ...
   ```

2. Add documentation maintenance rules:
   ```
   ### After Changes:
   | Change Type | Update File |
   |------------|-------------|
   | New module | `crate/agents/docs/<module>/index.yaml` |
   | New types/traits | `crate/agents/docs/index.yaml` exported_items |
   | Module removal | Delete `crate/agents/docs/<module>/` |
   ```

3. Document MCP tool usage for crate docs

**Files to modify:** `.agents/instructions.md`, `AGENTS.md`

---

### Issue 13: Update README

**Priority:** Medium  
**Type:** Documentation

Update `tools/mcp-docs-server/README.md`:
- Document new crate documentation tools
- Add usage examples for crate browsing/updating
- Update VS Code configuration if needed

**Files to modify:** `tools/mcp-docs-server/README.md`

---

## Implementation Order

1. **Phase 1 - Core Infrastructure**
   - Issue 1: Schema design
   - Issue 2: YAML parser
   - Issue 3: Crate discovery

2. **Phase 2 - Read Operations**
   - Issue 4: list_crates
   - Issue 5: browse_crate
   - Issue 6: read_crate_doc
   - Issue 9: search_crate_docs

3. **Phase 3 - Write Operations**
   - Issue 7: update_crate_doc
   - Issue 8: create_module_doc

4. **Phase 4 - Validation & Docs**
   - Issue 10: validate_crate_docs
   - Issue 12: Update instructions
   - Issue 13: Update README

5. **Phase 5 - Nice-to-have**
   - Issue 11: sync_crate_docs (auto-generation)

---

## Open Questions

1. **Unified search?** Should `search_docs` search both agent docs AND crate docs, or keep them separate?
   - Recommendation: Add `include_crate_docs: bool` parameter to existing search

2. **Environment variable?** Current tool uses `AGENTS_DIR`. Add `CRATES_DIR` or derive from parent?
   - Recommendation: Derive from `AGENTS_DIR.parent()/crates`

3. **Index regeneration?** Should there be a master index for all crates?
   - Recommendation: Not needed initially - each crate self-contained

4. **Version tracking?** Should crate docs track version changes?
   - Recommendation: Defer - just use crate version from Cargo.toml

---

## Notes

- Keep backward compatibility with existing agent docs tools
- All crate doc tools should be prefixed/grouped for discoverability (e.g., `crate_*` or separate namespace)
- Consider adding MCP resource support later for direct file access
