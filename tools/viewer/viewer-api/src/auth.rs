//! Bearer token authentication primitives shared across viewer tools.

use std::{
    collections::HashSet,
    sync::Arc,
};

use axum::{
    body::Body,
    http::{HeaderMap, Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::error::{ApiError, RequestIdExt};

/// A set of valid bearer tokens for in-memory validation.
///
/// Callers that need hot-reload should wrap this in an `ArcSwap<TokenSet>` and
/// swap it atomically on reload without rebuilding the middleware stack.
#[derive(Clone, Debug)]
pub struct TokenSet {
    tokens: HashSet<String>,
}

impl TokenSet {
    /// Build a `TokenSet` from an iterator of token strings.
    pub fn new(tokens: impl IntoIterator<Item = String>) -> Self {
        Self {
            tokens: tokens.into_iter().collect(),
        }
    }

    /// Convenience constructor for a single token.
    pub fn single(token: impl Into<String>) -> Self {
        let mut set = HashSet::new();
        set.insert(token.into());
        Self { tokens: set }
    }

    /// Returns `true` if `token` is a member of this set.
    pub fn contains(&self, token: &str) -> bool {
        self.tokens.contains(token)
    }

    /// Number of tokens in the set.
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

/// Extract the raw bearer token from an `Authorization` header, if present.
///
/// Returns `None` if the header is missing or not a `Bearer` scheme.
pub fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get("authorization")?.to_str().ok()?;
    value.strip_prefix("Bearer ").map(str::trim)
}

/// Axum middleware that validates the `Authorization: Bearer <token>` header
/// against an `Arc<TokenSet>` stored in the request extensions.
///
/// # Usage
/// ```rust,ignore
/// let token_set = Arc::new(TokenSet::single("secret"));
/// let app = Router::new()
///     .route("/api/tickets", get(list_tickets))
///     .layer(axum::middleware::from_fn_with_state(token_set, bearer_auth_mw));
/// ```
pub async fn bearer_auth_mw(
    axum::extract::State(token_set): axum::extract::State<Arc<TokenSet>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let request_id = request
        .extensions()
        .get::<RequestIdExt>()
        .map(|r| r.0.clone())
        .unwrap_or_default();

    match extract_bearer_token(request.headers()) {
        Some(token) if token_set.contains(token) => next.run(request).await,
        Some(_) => ApiError::unauthorized("auth.invalid_token", "Bearer token is invalid", &request_id)
            .into_response_with_status(StatusCode::UNAUTHORIZED),
        None => ApiError::unauthorized("auth.missing_token", "Authorization header required", &request_id)
            .into_response_with_status(StatusCode::UNAUTHORIZED),
    }
}
