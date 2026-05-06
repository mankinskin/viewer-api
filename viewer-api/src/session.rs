//! Session management for viewer applications.
//!
//! This module provides per-client session tracking with configurable settings.
//! Sessions are identified by the `x-session-id` HTTP header.
//!
//! # Example
//!
//! ```rust,no_run
//! use viewer_api::session::{SessionStore, SessionConfig, SESSION_HEADER, get_session_config};
//! use std::sync::Arc;
//!
//! let sessions = SessionStore::default();
//! // ... use in handlers
//! ```

use crate::axum::http::HeaderMap;
use serde::{
    Deserialize,
    Serialize,
};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        RwLock,
    },
};

/// Session configuration for per-client behavior.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Unique session identifier
    pub session_id: String,
    /// Whether to enable verbose logging for this session (default: false)
    #[serde(default)]
    pub verbose: bool,
    /// Custom session data (viewer-specific)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub data: HashMap<String, String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            verbose: false,
            data: HashMap::new(),
        }
    }
}

impl SessionConfig {
    /// Create a new session with the given ID
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            ..Default::default()
        }
    }
}

/// Session store - maps session IDs to their configuration.
/// Thread-safe and shareable across handlers.
#[derive(Clone, Default)]
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<String, SessionConfig>>>,
}

impl SessionStore {
    /// Create a new empty session store
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create a session by ID
    pub fn get_or_create(
        &self,
        session_id: &str,
    ) -> SessionConfig {
        // Try read first
        {
            let guard = self.sessions.read().unwrap();
            if let Some(config) = guard.get(session_id) {
                return config.clone();
            }
        }

        // Create new session
        let config = SessionConfig::new(session_id);
        let mut guard = self.sessions.write().unwrap();
        guard
            .entry(session_id.to_string())
            .or_insert_with(|| config.clone());
        config
    }

    /// Get a session if it exists
    pub fn get(
        &self,
        session_id: &str,
    ) -> Option<SessionConfig> {
        let guard = self.sessions.read().unwrap();
        guard.get(session_id).cloned()
    }

    /// Update a session's configuration
    pub fn update<F>(
        &self,
        session_id: &str,
        f: F,
    ) -> Option<SessionConfig>
    where
        F: FnOnce(&mut SessionConfig),
    {
        let mut guard = self.sessions.write().unwrap();
        if let Some(config) = guard.get_mut(session_id) {
            f(config);
            Some(config.clone())
        } else {
            None
        }
    }

    /// Set a data value for a session
    pub fn set_data(
        &self,
        session_id: &str,
        key: &str,
        value: &str,
    ) -> bool {
        let mut guard = self.sessions.write().unwrap();
        if let Some(config) = guard.get_mut(session_id) {
            config.data.insert(key.to_string(), value.to_string());
            true
        } else {
            false
        }
    }

    /// Get a data value from a session
    pub fn get_data(
        &self,
        session_id: &str,
        key: &str,
    ) -> Option<String> {
        let guard = self.sessions.read().unwrap();
        guard.get(session_id).and_then(|c| c.data.get(key).cloned())
    }
}

/// Header name for session identification
pub const SESSION_HEADER: &str = "x-session-id";

/// Get the session ID from request headers
pub fn get_session_id(headers: &HeaderMap) -> Option<&str> {
    headers.get(SESSION_HEADER).and_then(|v| v.to_str().ok())
}

/// Get or create session config from headers.
/// Returns None if no session ID header is present.
pub fn get_session_config(
    store: &SessionStore,
    headers: &HeaderMap,
) -> Option<SessionConfig> {
    let session_id = get_session_id(headers)?;
    Some(store.get_or_create(session_id))
}

/// Request type for updating session configuration
#[derive(Clone, Debug, Deserialize)]
pub struct SessionConfigUpdate {
    /// Update verbose logging setting
    #[serde(default)]
    pub verbose: Option<bool>,
    /// Update custom data values
    #[serde(default)]
    pub data: Option<HashMap<String, String>>,
}
