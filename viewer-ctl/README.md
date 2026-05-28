<!-- rule-api:file generated=true -->

<!-- rule-api:entry id=f796a0b0-33f6-4428-9647-071f3f98b352 slug=viewer-api/readme/tools/viewer-ctl/l1 -->
# viewer-ctl

CLI lifecycle manager for viewer servers and their linked frontend bundles.

## Interface

Use `viewer-ctl` when you need to inspect configured viewer components, build or install their artifacts, or manage running viewer instances declared in `viewer-ctl.toml`.

Primary commands:

- `list`: show every configured viewer component.
- `status`: inspect whether one or all managed servers are running.
- `build`, `install`: build artifacts and install them into the managed runtime layout.
- `start`, `stop`, `restart`: manage a server process on its configured port.
- `prepare`: build and install the frontend linked to a server before launch.
- `task`: run a named task from the configuration.
- `static-dir`: print the resolved static asset directory for a server's linked frontend.

Notable command behavior:

- `start <SERVER>` accepts `--foreground` and forwards any extra arguments after `--` to the server binary.
- `prepare <SERVER>` is designed for editor and debug pre-launch flows; it prints the resolved install directory to stdout.
- Failing `build` or `prepare` steps report the executed command, working directory, exit status, and captured child output tail so wrappers around `viewer-ctl` still receive actionable diagnostics.
- `static-dir <SERVER>` does not build anything; it only resolves the current install location.

## Usage

Run from a checkout that contains `viewer-ctl.toml`:

```bash
cargo run -p viewer-ctl -- --help
```

`viewer-ctl` reads the component definitions from `viewer-ctl.toml` and manages the installed server/frontend artifacts for those named components.

## Examples

```bash
# List the configured viewer components
viewer-ctl list

# Build and install the frontend for one managed server
viewer-ctl prepare ticket-viewer

# Start a viewer and keep logs attached to the current terminal
viewer-ctl start --foreground ticket-viewer

# Forward extra args to the server binary after --
viewer-ctl start spec-viewer -- --port 4010

# Resolve the current static bundle directory for a managed server
viewer-ctl static-dir ticket-viewer
```
