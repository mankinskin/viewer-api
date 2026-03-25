//! Structured API error envelope matching the api-contract-v0.1.md error shape.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use serde_json::Value;

/// Extension key type for request IDs injected by the request-id middleware.
#[derive(Clone, Debug, Default)]
pub struct RequestIdExt(pub String);

/// Structured error envelope returned on all 4xx/5xx responses.
///
/// Shape matches api-contract-v0.1.md:
/// ```json
/// {
///   "code": "auth.invalid_token",
///   "message": "Bearer token is invalid",
///   "request_id": "...",
///   "details": {}
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl ApiError {
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        request_id: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            request_id: request_id.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Build a 401 Unauthorized error.
    pub fn unauthorized(
        code: impl Into<String>,
        message: impl Into<String>,
        request_id: &str,
    ) -> Self {
        Self::new(code, message, request_id)
    }

    /// Build a 404 Not Found error.
    pub fn not_found(
        resource: impl Into<String>,
        request_id: &str,
    ) -> Self {
        let resource = resource.into();
        Self::new(
            "not_found",
            format!("{resource} not found"),
            request_id,
        )
    }

    /// Build a 400 Bad Request error.
    pub fn bad_request(
        code: impl Into<String>,
        message: impl Into<String>,
        request_id: &str,
    ) -> Self {
        Self::new(code, message, request_id)
    }

    /// Build a 500 Internal Server Error.
    pub fn internal(request_id: &str) -> Self {
        Self::new(
            "internal_error",
            "An unexpected error occurred",
            request_id,
        )
    }

    /// Build a 409 Conflict error.
    pub fn conflict(
        code: impl Into<String>,
        message: impl Into<String>,
        request_id: &str,
    ) -> Self {
        Self::new(code, message, request_id)
    }

    /// Render as an axum Response with the given status code.
    pub fn into_response_with_status(self, status: StatusCode) -> Response {
        (status, Json(self)).into_response()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Default to 500; callers should use into_response_with_status for
        // specific codes.
        self.into_response_with_status(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
