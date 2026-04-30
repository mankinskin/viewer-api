//! [`PathCodec`] — encode/decode hierarchical IDs ↔ URL path segments.
//!
//! The default [`ColonSegmented`] encoder mirrors the spec/crate viewer
//! convention: `category:name:sub/path` where the leading colon-separated
//! segments identify a node and the trailing slash-separated tail addresses
//! a sub-resource (e.g. a file inside a crate module).
//!
//! All operations are pure and free of `web_sys` — safe to unit-test on
//! native targets.

use std::collections::HashSet;

/// A bidirectional codec between a logical id (e.g. "spec:auth:login")
/// and a URL-friendly path (e.g. "spec/auth/login").
///
/// Implementations should be pure and total: `decode(encode(id)) == Some(id)`
/// for every id the codec accepts.
pub trait PathCodec {
    /// Encode a logical id into a URL path fragment.
    fn encode(&self, id: &str) -> String;

    /// Decode a URL path fragment back into a logical id.
    ///
    /// Returns `None` if the path is not a valid encoding for this codec.
    fn decode(&self, path: &str) -> Option<String>;
}

/// Default codec: replaces ':' with '/' on encode, '/' with ':' on decode.
///
/// Empty segments are preserved; leading/trailing separators are kept.
/// Round-trips losslessly for inputs that use only one separator alphabet.
#[derive(Clone, Copy, Debug, Default)]
pub struct ColonSegmented;

impl PathCodec for ColonSegmented {
    fn encode(&self, id: &str) -> String {
        id.replace(':', "/")
    }

    fn decode(&self, path: &str) -> Option<String> {
        if path.is_empty() {
            return None;
        }
        Some(path.replace('/', ":"))
    }
}

/// Inserts every prefix of `segments` (joined by `:`) into `set`.
///
/// Useful for syncing a tree-view's expanded-node set to a URL path so the
/// path of a deeply nested selection auto-expands its ancestors.
///
/// ```
/// use std::collections::HashSet;
/// use viewer_api_dioxus::store::expand_path_to;
///
/// let mut set = HashSet::new();
/// expand_path_to(&mut set, &["spec", "auth", "login"]);
/// assert!(set.contains("spec"));
/// assert!(set.contains("spec:auth"));
/// assert!(set.contains("spec:auth:login"));
/// ```
pub fn expand_path_to(set: &mut HashSet<String>, segments: &[&str]) {
    for i in 1..=segments.len() {
        set.insert(segments[..i].join(":"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colon_segmented_roundtrip() {
        let c = ColonSegmented;
        let id = "spec:auth:login";
        let p = c.encode(id);
        assert_eq!(p, "spec/auth/login");
        assert_eq!(c.decode(&p), Some(id.to_string()));
    }

    #[test]
    fn colon_segmented_decode_empty() {
        assert_eq!(ColonSegmented.decode(""), None);
    }

    #[test]
    fn colon_segmented_single_segment() {
        let c = ColonSegmented;
        assert_eq!(c.encode("spec"), "spec");
        assert_eq!(c.decode("spec"), Some("spec".to_string()));
    }

    #[test]
    fn expand_path_to_inserts_all_prefixes() {
        let mut set = HashSet::new();
        expand_path_to(&mut set, &["a", "b", "c"]);
        assert_eq!(set.len(), 3);
        assert!(set.contains("a"));
        assert!(set.contains("a:b"));
        assert!(set.contains("a:b:c"));
    }

    #[test]
    fn expand_path_to_empty_segments_is_noop() {
        let mut set: HashSet<String> = HashSet::new();
        expand_path_to(&mut set, &[]);
        assert!(set.is_empty());
    }
}
