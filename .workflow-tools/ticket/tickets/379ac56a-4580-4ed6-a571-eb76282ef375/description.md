## Defect

The viewer-api remote at SHA `52456b47` declares an in-tree dependency in `viewer-api/Cargo.toml` (local superproject path: `viewer-api/viewer-api/Cargo.toml`):

```toml
context-api = { path = "../../context-stack/context-api" }
```

A standalone clone of `https://github.com/mankinskin/viewer-api` has no `context-stack/context-api` package at that relative path, so Cargo consumers cannot resolve viewer-api.

## Reproduction

1. Start from a clean clone of `https://github.com/mankinskin/viewer-api` at remote SHA `52456b47`.
2. Run a standalone viewer-api build or test command that resolves the manifest, such as `cargo check`.
3. Alternatively, build extracted ticket repository `ba4aaa9c`; its `cargo check` resolves viewer-api as an external dependency.

## Observed behavior

Cargo fails dependency resolution with `no matching package named context-api` before the standalone viewer-api build can proceed. The extracted `ba4aaa9c` ticket-viewer build reproduces the same failure during viewer-api resolution.

## Expected behavior

viewer-api must be independently consumable from a clean clone. Its `context-api` dependency must resolve from the canonical remote `https://github.com/mankinskin/context-stack`, branch `main`, so downstream tool repositories can progress past viewer-api dependency resolution.

Expectation source: completed ticket `1c452ff1` states that viewer-api is a standalone shared dependency consumable by every per-tool repository; related ticket `ba4aaa9c` depends on that property while building the extracted ticket tool.

## Root cause

The manifest encodes a relative path into the context-engine superproject rather than the canonical external context-stack dependency. The path is valid only inside the superproject checkout and absent from the viewer-api remote tree.

## Acceptance criteria

- `viewer-api/Cargo.toml` uses the canonical context-stack git dependency: `https://github.com/mankinskin/context-stack`, branch `main`, with Cargo feature syntax adjusted as required.
- A clean standalone viewer-api clone builds and tests successfully.
- `ba4aaa9c` extracted ticket repository `cargo check` proceeds past viewer-api dependency resolution.
- viewer-api `main` is pushed and the context-engine viewer-api gitlink is updated when the superproject still needs the new revision.

## Scope and evidence

- Owner: viewer-api repository.
- Component: viewer-api.
- Suspected manifest: `viewer-api/viewer-api/Cargo.toml` in context-engine; `viewer-api/Cargo.toml` in the standalone remote.
- Environment: Windows; Cargo failure reproduced by the `ba4aaa9c` extracted ticket-viewer build.
- No source change, implementation plan, state transition, or closure is included in this bug capture.

## Traceability

- Shared contract spec: `5ee7f36a` (Workflow-tools domain crate contract).
- Related tickets: `ba4aaa9c` (extracted ticket tool build) and completed `1c452ff1` (viewer-api standalone dependency foundations).


## Implementation plan

1. In the standalone `viewer-api` repository, change only `viewer-api/Cargo.toml`: replace the superproject-relative `context-api` path dependency with `context-api = { git = "https://github.com/mankinskin/context-stack", branch = "main" }`, preserving required feature syntax.
2. Do not use a committed local path or superproject `[patch]` workaround. The external dependency policy is settled by the shared domain-crate contract; this manifest-only defect introduces no new type or trait ownership.
3. Run the required clean-clone and downstream checks. Push the upstream `viewer-api` revision, then update the context-engine `viewer-api` gitlink only when the superproject needs that revision.

## Required validation

```bash
standalone_dir="$(mktemp -d)"
git clone --depth 1 --branch main https://github.com/mankinskin/viewer-api "$standalone_dir/viewer-api"
cargo check --manifest-path "$standalone_dir/viewer-api/viewer-api/Cargo.toml"
cargo test --manifest-path "$standalone_dir/viewer-api/viewer-api/Cargo.toml"
rm -rf "$standalone_dir"

cargo check --manifest-path ticket/Cargo.toml --all-features
```

The first commands must run against a clean standalone clone. The final command must progress past `viewer-api` dependency resolution; a later `ba4aaa9c` task owns any remaining unrelated extraction failures.
