//! Axum middleware utilities shared across viewer tools.

pub mod request_id {
    //! Middleware that generates a `X-Request-Id` header per request and
    //! injects a `REQUEST_ID_EXT` extension for downstream handlers to reuse.

    use axum::{
        body::Body,
        http::Request,
        middleware::Next,
        response::Response,
    };
    use uuid::Uuid;

    use crate::error::RequestIdExt;

    /// Axum `from_fn` middleware: generates a UUID v4 request ID, injects it
    /// into request extensions, and echoes it back in `X-Request-Id` response
    /// header.
    ///
    /// # Usage
    /// ```rust,ignore
    /// let app = Router::new()
    ///     .route("/api/tickets", get(handler))
    ///     .layer(axum::middleware::from_fn(add_request_id));
    /// ```
    pub async fn add_request_id(
        mut request: Request<Body>,
        next: Next,
    ) -> Response {
        let id = Uuid::new_v4().to_string();
        request.extensions_mut().insert(RequestIdExt(id.clone()));

        let mut response = next.run(request).await;

        if let Ok(value) = axum::http::HeaderValue::from_str(&id) {
            response.headers_mut().insert("x-request-id", value);
        }

        response
    }
}
