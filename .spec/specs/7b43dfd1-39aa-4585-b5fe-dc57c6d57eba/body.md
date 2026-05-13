# viewer-api: icons + Spinner

Canonical specification for the shared icon set and `Spinner` Dioxus
components (`viewer-api/frontend/dioxus/src/components/icons.rs` and
`spinner.rs`).

## Public surface

- `Spinner { size: SpinnerSize }` with `SpinnerSize::{ Sm, Md, Lg }`.
- Icon components (each is a thin SVG-emitting Dioxus component):
  `AlertIcon`, `CheckIcon`, `ChevronDownIcon`, `ChevronRightIcon`,
  `CloseIcon`, `CodeIcon`, `CrateIcon`, `DocumentIcon`, `FileIcon`,
  `FilterIcon`, `FolderIcon`, `FolderOpenIcon`, `GraphIcon`,
  `HamburgerIcon`, `HomeIcon`, `InfoIcon`, `LogIcon`, `MinusIcon`,
  `ModuleIcon`, `PlusIcon`, `RefreshIcon`, `SearchIcon`, `SourceFileIcon`,
  `StatsIcon`.

## Demo behavior

The `pages/icons_spinner.rs` page renders:

1. A grid of every icon with its name as a caption and at three sizes
   (16 / 24 / 32 px).
2. The three `Spinner` sizes side by side, plus a "spinner-on-glass"
   variant inside a `GlassPanel`.
3. A click-to-copy: clicking an icon copies its Dioxus tag
   (e.g. `FolderOpenIcon { class: "..." }`) to the clipboard.

## Acceptance behavior (validated by e2e)

- Every icon component renders an `<svg>` element with non-zero bounding
  box at all three sizes.
- The page renders without console errors.
- `Spinner` rotates (CSS animation): a snapshot taken at t=0 ms differs
  from a snapshot at t=300 ms (verified via `expect(locator).toHaveScreenshot`
  with animation `allow`).

## Code references

- `tools/viewer/viewer-api/frontend/dioxus/src/components/icons.rs`
- `tools/viewer/viewer-api/frontend/dioxus/src/components/spinner.rs`
- `tools/viewer/e2e/tests/demo-viewer/icons-spinner.spec.ts`
