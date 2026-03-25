#!/usr/bin/env bash
# Unified launcher for viewer MCP servers.
# Builds the release binary if it doesn't exist or sources have changed,
# then exec's into it.
#
# Usage:
#   run-mcp.sh <tool-name> [args...]
#
# Examples:
#   run-mcp.sh doc-viewer --mcp
#   run-mcp.sh log-viewer --mcp
#
# Designed to be used as the MCP server command in Zed/VS Code settings.

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: run-mcp.sh <tool-name> [args...]" >&2
    echo "Available tools: doc-viewer, log-viewer" >&2
    exit 1
fi

TOOL_NAME="$1"
shift

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TOOLS_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
WORKSPACE_ROOT="$(cd "$TOOLS_DIR/../../.." && pwd)"
TOOL_DIR="$WORKSPACE_ROOT/tools/$TOOL_NAME"

# Validate tool exists — search flat and layer-nested locations
if [[ ! -f "$TOOL_DIR/Cargo.toml" ]]; then
    # Try layer subdirectories: cli/, http/, mcp/, viewer/
    FOUND=""
    for layer in cli http mcp viewer; do
        CANDIDATE="$WORKSPACE_ROOT/tools/$layer/$TOOL_NAME"
        if [[ -f "$CANDIDATE/Cargo.toml" ]]; then
            FOUND="$CANDIDATE"
            break
        fi
    done
    if [[ -z "$FOUND" ]]; then
        echo "ERROR: Unknown tool '$TOOL_NAME' — no Cargo.toml found" >&2
        echo "Searched: $WORKSPACE_ROOT/tools/$TOOL_NAME and tools/{cli,http,mcp,viewer}/$TOOL_NAME" >&2
        exit 1
    fi
    TOOL_DIR="$FOUND"
fi
TOOL_DIR="$(cd "$TOOL_DIR" && pwd)"

# Binary lives in the workspace-level target dir (workspace member build)
if [[ -f "$WORKSPACE_ROOT/target/release/$TOOL_NAME.exe" ]] \
    || [[ "$(uname -s)" == *MINGW* ]] \
    || [[ "$(uname -s)" == *MSYS* ]]; then
    EXE="$WORKSPACE_ROOT/target/release/$TOOL_NAME.exe"
else
    EXE="$WORKSPACE_ROOT/target/release/$TOOL_NAME"
fi

build() {
    echo "Building $TOOL_NAME (release)..." >&2
    cargo build --release --manifest-path "$TOOL_DIR/Cargo.toml" >&2
}

# Build if binary doesn't exist
if [[ ! -f "$EXE" ]]; then
    echo "$TOOL_NAME binary not found at $EXE" >&2
    build
fi

# Rebuild if any source file is newer than the binary
if [[ -d "$TOOL_DIR/src" ]]; then
    NEWER=$(find "$TOOL_DIR/src" -name '*.rs' -newer "$EXE" 2>/dev/null | head -1)
    if [[ -n "$NEWER" ]]; then
        echo "Source files changed since last build (e.g. $NEWER)" >&2
        build
    fi
fi

# Also rebuild if shared viewer-api sources changed
VIEWER_API_SRC="$WORKSPACE_ROOT/tools/viewer/viewer-api/src"
if [[ -d "$VIEWER_API_SRC" ]]; then
    NEWER=$(find "$VIEWER_API_SRC" -name '*.rs' -newer "$EXE" 2>/dev/null | head -1)
    if [[ -n "$NEWER" ]]; then
        echo "viewer-api sources changed since last build (e.g. $NEWER)" >&2
        build
    fi
fi

if [[ ! -f "$EXE" ]]; then
    echo "ERROR: Build failed — $EXE not found" >&2
    exit 1
fi

exec "$EXE" "$@"
