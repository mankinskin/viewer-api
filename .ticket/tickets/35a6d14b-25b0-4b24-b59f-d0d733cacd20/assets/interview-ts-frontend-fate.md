# Interview: TypeScript Frontend Fate

**Date:** 2026-04-08
**Applies to:** Epic `35a6d14b`

## Question

After the Dioxus port reaches parity: remove the TypeScript frontend entirely, keep as fallback, or freeze?

## Answer

**Archive/freeze it and eventually remove it after the new frontend is tested and progressed to a proven product.**

## Implications

- TypeScript frontend (viewer-api/frontend + ticket-viewer/frontend) enters freeze immediately
- No new features in TypeScript — all new development goes to Dioxus
- Keep TypeScript frontend runnable as a fallback during the Dioxus transition period
- Once Dioxus frontend reaches feature parity and passes acceptance testing, archive the TypeScript code
- Final removal after a proving period (duration TBD)
- Consider adding a "legacy" flag to the build system that still builds the TS frontend
