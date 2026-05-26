## Single source

Per-viewer crates must not duplicate the contract types or re-implement the shared frontend logic. If a viewer needs a behaviour that is not covered, the behaviour is added to `viewer-api` first and then consumed by every viewer in lockstep.