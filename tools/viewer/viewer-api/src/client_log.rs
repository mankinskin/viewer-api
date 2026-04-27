//! `POST /api/client-log` — receive browser tracing records and append them
//! to a JSONL file on disk so that log-viewer MCP tools can query them
//! alongside server-side logs.
//!
//! # Usage
//!
//! Mount [`client_log_router`] into any viewer's axum `Router`:
//!
//! ```rust,no_run
//! use viewer_api::client_log::{client_log_router, ClientLogState};
//!
//! let log_state = ClientLogState::default();
//! let app = your_api_router.merge(client_log_router(log_state));
//! ```
//!
//! # File location
//!
//! Default path: `target/test-logs/frontend-client.jsonl` relative to the
//! current working directory.  Override with [`ClientLogState::with_path`].
//!
//! # Request format
//!
//! ```json
//! { "records": [ { "ts": 1234567890, "level": "info", "target": "...", ... } ] }
//! ```
//!
//! # Security
//!
//! The endpoint accepts requests from `127.0.0.1` only (enforced by the
//! caller — typically `viewer-ctl` binds to loopback).  No auth header is
//! required given the loopback-only binding.  Body size is capped at 1 MiB.

use axum::{
    body::Bytes,
    extract::State,
    http::StatusCode,
    routing::post,
    Router,
};
use serde::Deserialize;
use std::{io::Write, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use tracing::warn;

const MAX_BODY_BYTES: usize = 1_048_576; // 1 MiB

// ── State ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ClientLogState {
    file_path: PathBuf,
    /// Serialises concurrent writes to the same file.
    write_lock: Arc<Mutex<()>>,
}

impl Default for ClientLogState {
    fn default() -> Self {
        let path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("target/test-logs/frontend-client.jsonl");
        Self::with_path(path)
    }
}

impl ClientLogState {
    /// Create a state that writes to `file_path`.  Parent directories are
    /// created eagerly so the first POST doesn't fail on a missing dir.
    pub fn with_path(file_path: impl Into<PathBuf>) -> Self {
        let file_path = file_path.into();
        if let Some(parent) = file_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        ClientLogState {
            file_path,
            write_lock: Arc::new(Mutex::new(())),
        }
    }
}

// ── Payload ───────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct IngestPayload {
    records: Vec<serde_json::Value>,
}

// ── Handler ───────────────────────────────────────────────────────────────────

async fn ingest(
    State(state): State<ClientLogState>,
    body: Bytes,
) -> Result<StatusCode, (StatusCode, String)> {
    if body.len() > MAX_BODY_BYTES {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, "body exceeds 1 MiB limit".into()));
    }

    let payload: IngestPayload = serde_json::from_slice(&body)
        .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()))?;

    if payload.records.is_empty() {
        return Ok(StatusCode::NO_CONTENT);
    }

    // Append records to the JSONL file.  We hold the async lock across the
    // blocking write so concurrent POSTs don't interleave lines.
    let _guard = state.write_lock.lock().await;
    let result = tokio::task::spawn_blocking({
        let path = state.file_path.clone();
        let records = payload.records;
        move || -> std::io::Result<()> {
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)?;
            for record in &records {
                let line = serde_json::to_string(record)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                writeln!(file, "{line}")?;
            }
            Ok(())
        }
    })
    .await
    .map_err(|e| {
        warn!(error = %e, "spawn_blocking panicked in client-log handler");
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    result.map_err(|e| {
        warn!(file = %state.file_path.display(), error = %e, "failed to write client log");
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Router factory ────────────────────────────────────────────────────────────

/// Build a sub-router that serves `POST /api/client-log`.
pub fn client_log_router(state: ClientLogState) -> Router {
    Router::new()
        .route("/api/client-log", post(ingest))
        .with_state(state)
}
