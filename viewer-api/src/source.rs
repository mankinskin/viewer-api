//! Source file utilities for resolving and serving source code.
//!
//! Provides common functions for source file resolution and language detection,
//! shared between viewer tools that need to display source code.

use std::path::PathBuf;

/// Strategy for resolving and fetching source files.
///
/// In a local development environment, source files are served directly from the
/// workspace on disk.  When deployed (e.g. in GitHub Actions / GitHub Pages), the
/// workspace is not available so files are fetched from the public repository using
/// the raw GitHub content API.
#[derive(Debug, Clone)]
pub enum SourceBackend {
    /// Read source files from the local filesystem.
    Local {
        workspace_root: PathBuf,
    },
    /// Fetch source files from a remote repository via raw HTTP.
    ///
    /// `raw_base_url` is the prefix that, when joined with a relative source path,
    /// produces a directly-downloadable URL.  For GitHub this looks like:
    /// `https://raw.githubusercontent.com/{owner}/{repo}/{commit}`.
    ///
    /// `source_tree_path` is an optional sub-directory within the repository that
    /// corresponds to the workspace root (e.g. when the workspace lives in a
    /// sub-folder of the repository).
    Remote {
        raw_base_url: String,
        source_tree_path: Option<String>,
    },
}

impl SourceBackend {
    /// Create a `Remote` backend pointing at a GitHub repository.
    pub fn github(
        owner: &str,
        repo: &str,
        commit: &str,
        source_tree_path: Option<String>,
    ) -> Self {
        let raw_base_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}",
            owner, repo, commit
        );
        Self::Remote {
            raw_base_url,
            source_tree_path,
        }
    }

    /// Automatically select the backend based on the runtime environment.
    ///
    /// When `GITHUB_ACTIONS=true`, `GITHUB_REPOSITORY`, and `GITHUB_SHA` are set
    /// (as they are in every GitHub Actions workflow), a `Remote` backend is
    /// returned so the server fetches files from the public repository.
    ///
    /// Otherwise (local development) the `Local` backend is used.
    pub fn detect(workspace_root: PathBuf) -> Self {
        if std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true") {
            if let (Ok(repo), Ok(sha)) = (
                std::env::var("GITHUB_REPOSITORY"),
                std::env::var("GITHUB_SHA"),
            ) {
                let raw_base_url = format!(
                    "https://raw.githubusercontent.com/{}/{}",
                    repo, sha
                );
                return Self::Remote {
                    raw_base_url,
                    source_tree_path: None,
                };
            }
        }
        Self::Local { workspace_root }
    }

    /// Build the URL (remote) or absolute path (local) for a given relative source path.
    ///
    /// Returns `Err` if the path is invalid (traversal attempt, etc.).
    pub fn resolve_url_or_path(&self, path: &str) -> Result<SourceLocation, String> {
        match self {
            Self::Local { workspace_root } => {
                let full_path = resolve_source_path(workspace_root, path)?;
                Ok(SourceLocation::Path(full_path))
            }
            Self::Remote {
                raw_base_url,
                source_tree_path,
            } => {
                // Sanitize: reject traversal attempts
                let normalized = path.replace('\\', "/");
                let clean = normalized.trim_start_matches('/');
                if clean.contains("..") {
                    return Err("Path traversal not allowed".to_string());
                }
                let url = if let Some(tree) = source_tree_path {
                    format!(
                        "{}/{}/{}",
                        raw_base_url,
                        tree.trim_end_matches('/'),
                        clean
                    )
                } else {
                    format!("{}/{}", raw_base_url, clean)
                };
                Ok(SourceLocation::Url(url))
            }
        }
    }
}

/// Resolved location for a source file – either a local path or a remote URL.
#[derive(Debug, Clone)]
pub enum SourceLocation {
    Path(PathBuf),
    Url(String),
}

/// Detect programming language from file extension.
///
/// Returns a language identifier string suitable for syntax highlighting.
pub fn detect_language(path: &str) -> String {
    let ext = path.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "json" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "md" => "markdown",
        "html" => "html",
        "css" => "css",
        _ => "plaintext",
    }
    .to_string()
}

/// Sanitize and resolve a source path relative to a workspace root.
///
/// Normalizes path separators, strips leading slashes, and checks for
/// path traversal attacks.
pub fn resolve_source_path(
    workspace_root: &PathBuf,
    path: &str,
) -> Result<PathBuf, String> {
    // Normalize path separators
    let normalized = path.replace('\\', "/");

    // Remove leading slashes
    let clean_path = normalized.trim_start_matches('/');

    // Check for path traversal
    if clean_path.contains("..") {
        return Err("Path traversal not allowed".to_string());
    }

    let full_path = workspace_root.join(clean_path);

    // Verify the path is within workspace
    if !full_path.starts_with(workspace_root) {
        return Err("Path outside workspace".to_string());
    }

    Ok(full_path)
}

/// Extract a snippet of lines from source content.
///
/// Returns (snippet, start_line, end_line) where lines are 1-based.
/// `context` is the number of lines to include above and below `line`.
pub fn extract_snippet(
    content: &str,
    line: usize,
    context: usize,
) -> (String, usize, usize) {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let line = line.min(total_lines).max(1);
    let start_line = line.saturating_sub(context).max(1);
    let end_line = (line + context).min(total_lines);

    let snippet_lines: Vec<&str> = lines[(start_line - 1)..end_line].to_vec();
    let snippet_content = snippet_lines.join("\n");

    (snippet_content, start_line, end_line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("main.rs"), "rust");
        assert_eq!(detect_language("index.ts"), "typescript");
        assert_eq!(detect_language("app.tsx"), "typescript");
        assert_eq!(detect_language("script.js"), "javascript");
        assert_eq!(detect_language("config.json"), "json");
        assert_eq!(detect_language("Cargo.toml"), "toml");
        assert_eq!(detect_language("config.yaml"), "yaml");
        assert_eq!(detect_language("config.yml"), "yaml");
        assert_eq!(detect_language("readme.md"), "markdown");
        assert_eq!(detect_language("page.html"), "html");
        assert_eq!(detect_language("style.css"), "css");
        assert_eq!(detect_language("Makefile"), "plaintext");
    }

    #[test]
    fn test_resolve_source_path_normal() {
        let root = PathBuf::from("/workspace");
        let result = resolve_source_path(&root, "src/main.rs");
        assert_eq!(result.unwrap(), Path::new("/workspace/src/main.rs"));
    }

    #[test]
    fn test_resolve_source_path_strips_leading_slash() {
        let root = PathBuf::from("/workspace");
        let result = resolve_source_path(&root, "/src/main.rs");
        assert_eq!(result.unwrap(), Path::new("/workspace/src/main.rs"));
    }

    #[test]
    fn test_resolve_source_path_normalizes_backslashes() {
        let root = PathBuf::from("/workspace");
        let result = resolve_source_path(&root, "src\\lib\\mod.rs");
        assert_eq!(result.unwrap(), Path::new("/workspace/src/lib/mod.rs"));
    }

    #[test]
    fn test_resolve_source_path_rejects_traversal() {
        let root = PathBuf::from("/workspace");
        let result = resolve_source_path(&root, "../etc/passwd");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Path traversal not allowed");
    }

    #[test]
    fn test_extract_snippet_middle() {
        let content = "line1\nline2\nline3\nline4\nline5\nline6\nline7";
        let (snippet, start, end) = extract_snippet(content, 4, 1);
        assert_eq!(snippet, "line3\nline4\nline5");
        assert_eq!(start, 3);
        assert_eq!(end, 5);
    }

    #[test]
    fn test_extract_snippet_start_clamped() {
        let content = "line1\nline2\nline3\nline4\nline5";
        let (snippet, start, end) = extract_snippet(content, 1, 2);
        assert_eq!(start, 1);
        assert_eq!(end, 3);
        assert_eq!(snippet, "line1\nline2\nline3");
    }

    #[test]
    fn test_extract_snippet_end_clamped() {
        let content = "line1\nline2\nline3\nline4\nline5";
        let (snippet, start, end) = extract_snippet(content, 5, 2);
        assert_eq!(start, 3);
        assert_eq!(end, 5);
        assert_eq!(snippet, "line3\nline4\nline5");
    }

    #[test]
    fn test_source_backend_local_resolves_path() {
        let root = PathBuf::from("/workspace");
        let backend = SourceBackend::Local { workspace_root: root };
        match backend.resolve_url_or_path("src/main.rs").unwrap() {
            SourceLocation::Path(p) => assert_eq!(p, Path::new("/workspace/src/main.rs")),
            SourceLocation::Url(_) => panic!("expected local path"),
        }
    }

    #[test]
    fn test_source_backend_local_rejects_traversal() {
        let root = PathBuf::from("/workspace");
        let backend = SourceBackend::Local { workspace_root: root };
        assert!(backend.resolve_url_or_path("../etc/passwd").is_err());
    }

    #[test]
    fn test_source_backend_remote_builds_url() {
        let backend = SourceBackend::Remote {
            raw_base_url: "https://raw.githubusercontent.com/owner/repo/abc123".to_string(),
            source_tree_path: None,
        };
        match backend.resolve_url_or_path("src/main.rs").unwrap() {
            SourceLocation::Url(url) => {
                assert_eq!(url, "https://raw.githubusercontent.com/owner/repo/abc123/src/main.rs");
            }
            SourceLocation::Path(_) => panic!("expected URL"),
        }
    }

    #[test]
    fn test_source_backend_remote_with_tree_path() {
        let backend = SourceBackend::Remote {
            raw_base_url: "https://raw.githubusercontent.com/owner/repo/abc123".to_string(),
            source_tree_path: Some("crates/my-crate".to_string()),
        };
        match backend.resolve_url_or_path("src/lib.rs").unwrap() {
            SourceLocation::Url(url) => {
                assert_eq!(url, "https://raw.githubusercontent.com/owner/repo/abc123/crates/my-crate/src/lib.rs");
            }
            SourceLocation::Path(_) => panic!("expected URL"),
        }
    }

    #[test]
    fn test_source_backend_remote_rejects_traversal() {
        let backend = SourceBackend::Remote {
            raw_base_url: "https://raw.githubusercontent.com/owner/repo/abc123".to_string(),
            source_tree_path: None,
        };
        assert!(backend.resolve_url_or_path("../etc/passwd").is_err());
    }

    #[test]
    fn test_source_backend_github_helper() {
        let backend = SourceBackend::github(
            "myowner",
            "myrepo",
            "deadbeef",
            Some("subdir".to_string()),
        );
        match backend.resolve_url_or_path("src/main.rs").unwrap() {
            SourceLocation::Url(url) => {
                assert_eq!(url, "https://raw.githubusercontent.com/myowner/myrepo/deadbeef/subdir/src/main.rs");
            }
            SourceLocation::Path(_) => panic!("expected URL"),
        }
    }
}
