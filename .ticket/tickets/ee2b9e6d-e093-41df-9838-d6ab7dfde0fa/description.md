# Manual validation epic

Final sign-off ticket. Closing this ticket transitions the umbrella spec
`viewer-api/demo-viewer` from `implemented` to `verified`.

## Checklist

- [ ] `viewer-ctl start demo-viewer` brings up the SPA on :3099.
- [ ] Every left-nav feature page renders without console errors.
- [ ] Every page's "Spec" link opens the matching spec in spec-viewer.
- [ ] `npx playwright test --project=demo-viewer` is green end-to-end
      on the developer's local Chromium.
- [ ] The WebGPU pages render correctly with the documented launch flags
      and gracefully degrade without them.
- [ ] The README accurately documents how to run and extend the
      demo-viewer.

## Output

- File a short note (5–10 lines) under `agents/implemented/` summarising
  the verification run + any deviations from the spec.
- Run `spec.exe update <demo-viewer-id> --state verified` once all
  intermediate states (`reviewed`, `approved`) are recorded.
