# Problem

The install contract now validates `viewer-ctl install` for all managed viewers, but `viewer-ctl` still has no first-class uninstall/remove command. That leaves `VIEW-04` as a manual gap in the install specification and blocks executable deinstall coverage for viewer artifacts.

# Scope

Implement a first-class uninstall command for `viewer-ctl` and wire it into the install contract.

The implementation should cover:

- uninstalling one managed viewer by name
- removing the installed server binary when `--kind server` is selected
- removing installed static assets when `--kind frontend` is selected
- defining behavior when only one of the artifacts is present
- documenting whether `viewer-ctl` itself is also uninstallable through the same surface or remains a separate `cargo uninstall viewer-ctl` path
- updating the canonical install contract in `.spec` to describe the supported viewer deinstall flow
- extending the Docker install validation harness to execute and assert the deinstall flow

# Follow-up Context

This is the implementation follow-up to design ticket `5d320d7e-f974-4d52-9e25-8265bf7a42cf` for install/deinstall validation.

# Acceptance Criteria

- `viewer-ctl` exposes a supported uninstall/remove command for managed viewers.
- The command removes the installed artifacts for the requested viewer kinds without damaging unrelated viewer installs.
- The canonical install contract records the supported uninstall flow and no longer treats viewer deinstall purely as an unspecified manual gap.
- The Docker viewer install-validation harness executes at least one uninstall path and verifies the expected artifacts are removed.
- User-facing documentation and contract-sync checks stay aligned with the supported uninstall behavior.
