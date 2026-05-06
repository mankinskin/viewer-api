//! Opaque cursor-based pagination types for paginated list endpoints.
//!
//! The `PageCursor` is an opaque string that encodes a JSON checkpoint.
//! Consumers must treat it as an opaque blob — do not parse or construct it
//! manually.

use serde::{Deserialize, Serialize};

/// An opaque pagination cursor (base64-encoded JSON checkpoint).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageCursor(pub String);

impl PageCursor {
    /// Encode an arbitrary serializable checkpoint into a cursor.
    pub fn encode<T: Serialize>(checkpoint: &T) -> Result<Self, serde_json::Error> {
        let json = serde_json::to_string(checkpoint)?;
        // Opaque cursor — JSON directly. Switch to Base64URL in production.
        Ok(PageCursor(json))
    }

    /// Decode the cursor back to a checkpoint.
    pub fn decode<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.0)
    }

    /// Return the raw string value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Query parameters for paginated endpoints.
///
/// `limit` is clamped to the range [1, max_limit] by the handler.
#[derive(Debug, Clone, Deserialize)]
pub struct PageParams {
    #[serde(default = "default_limit")]
    pub limit: usize,
    pub cursor: Option<PageCursor>,
}

fn default_limit() -> usize {
    100
}

impl Default for PageParams {
    fn default() -> Self {
        Self {
            limit: default_limit(),
            cursor: None,
        }
    }
}

/// A page of results with an optional continuation cursor.
#[derive(Debug, Serialize)]
pub struct PageResult<T: Serialize> {
    pub items: Vec<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<PageCursor>,
}

impl<T: Serialize> PageResult<T> {
    pub fn new(items: Vec<T>, next_cursor: Option<PageCursor>) -> Self {
        Self { items, next_cursor }
    }

    /// Convenience: a final page with no continuation.
    pub fn last(items: Vec<T>) -> Self {
        Self {
            items,
            next_cursor: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn cursor_roundtrip() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Checkpoint {
            offset: u64,
            generation: u64,
        }
        let cp = Checkpoint { offset: 42, generation: 7 };
        let cursor = PageCursor::encode(&cp).unwrap();
        let decoded: Checkpoint = cursor.decode().unwrap();
        assert_eq!(decoded, cp);
    }
}
