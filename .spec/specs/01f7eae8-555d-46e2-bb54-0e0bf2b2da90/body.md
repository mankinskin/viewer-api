# viewer-ctl/lifecycle/task

A task is an ordered list of shell command invocations. Tasks are the
escape hatch for repeatable multi-step pipelines that don't fit the
build/install/start verbs of any single component (e.g. type generation).

---

## Schema

See `viewer-ctl/config` for the TOML schema. Each step is a triple of
`(dir, cmd, allow_failure)`.

---

## Execution

```text
[<task.name>] <task.description>      (skipped if description is empty)
[<task.name>] step: <cmd> (cwd=<dir>)
... command output ...
[<task.name>] step: <cmd> (cwd=<dir>)
... command output ...
[<task.name>] done.
```

For each step:

1. Resolve `dir` relative to repo root.
2. Run `cmd` from that directory.
3. On non-zero exit:
   - If `allow_failure = true` → log a warning and continue.
   - Otherwise → abort the task and propagate the error.

---

## Idiom: `gen-types`

The canonical task is `gen-types`, which exports TypeScript bindings from
ts-rs annotations across multiple Rust crates and then runs the npm build
in `packages/context-types`. The Rust steps use `allow_failure = true`
because the test invocation generates the bindings as a side-effect even
when the test itself "fails" by being marked `#[ignore]`.

---

## Acceptance Criteria

- A task with zero steps prints a header and a `done.` line.
- An `allow_failure = false` step that exits non-zero stops the task and
  exits viewer-ctl with status 1.
- An `allow_failure = true` step that exits non-zero produces a warning and
  the task continues with the next step.
