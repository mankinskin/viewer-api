use axum::{
    body::Body,
    extract::Request,
    response::{
        IntoResponse,
        Response,
    },
};
use hyper::StatusCode;
use hyper_util::{
    client::legacy::Client,
    rt::{
        TokioExecutor,
        TokioIo,
    },
};
use tracing::{
    debug,
    error,
    warn,
};

pub(super) async fn proxy_websocket(
    mut req: Request,
    upstream_uri: hyper::Uri,
) -> Response {
    let Some(browser_upgrade) = take_browser_upgrade(&mut req) else {
        return (StatusCode::BAD_REQUEST, "Missing upgrade extension")
            .into_response();
    };

    let vite_resp = match forward_websocket_upgrade(req, upstream_uri).await {
        Ok(resp) => resp,
        Err(resp) => return resp,
    };

    if vite_resp.status() != StatusCode::SWITCHING_PROTOCOLS {
        warn!(
            status = %vite_resp.status(),
            "Vite did not accept WebSocket upgrade"
        );
        return vite_resp.into_response();
    }

    debug!("Vite accepted WebSocket upgrade, setting up bidirectional pipe");
    let browser_response = match build_upgrade_response(vite_resp.headers()) {
        Ok(resp) => resp,
        Err(resp) => return resp,
    };

    tokio::spawn(async move {
        bridge_websocket_io(vite_resp, browser_upgrade).await;
    });

    browser_response
}

fn take_browser_upgrade(
    req: &mut Request
) -> Option<hyper::upgrade::OnUpgrade> {
    let upgrade = req.extensions_mut().remove::<hyper::upgrade::OnUpgrade>();
    if upgrade.is_none() {
        error!("WebSocket request missing OnUpgrade extension");
    }
    upgrade
}

async fn forward_websocket_upgrade(
    req: Request,
    upstream_uri: hyper::Uri,
) -> Result<hyper::Response<hyper::body::Incoming>, Response> {
    let client = Client::builder(TokioExecutor::new()).build_http::<Body>();
    let (mut parts, body) = req.into_parts();
    parts.uri = upstream_uri;
    parts.headers.remove(hyper::header::HOST);

    let vite_req = Request::from_parts(parts, body);
    client.request(vite_req).await.map_err(|error| {
        error!(error = %error, "Failed to proxy WebSocket to Vite");
        (
            StatusCode::BAD_GATEWAY,
            format!("Dev proxy WebSocket error: {}", error),
        )
            .into_response()
    })
}

fn build_upgrade_response(
    headers: &hyper::HeaderMap
) -> Result<Response, Response> {
    let mut resp_builder =
        Response::builder().status(StatusCode::SWITCHING_PROTOCOLS);
    for (name, value) in headers {
        resp_builder = resp_builder.header(name, value);
    }

    resp_builder.body(Body::empty()).map_err(|error| {
        error!(error = %error, "Failed to build WebSocket upgrade response");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to build upgrade response",
        )
            .into_response()
    })
}

async fn bridge_websocket_io(
    vite_resp: hyper::Response<hyper::body::Incoming>,
    browser_upgrade: hyper::upgrade::OnUpgrade,
) {
    let vite_upgraded = match hyper::upgrade::on(vite_resp).await {
        Ok(io) => io,
        Err(error) => {
            error!(error = %error, "Vite WebSocket upgrade IO failed");
            return;
        },
    };

    let browser_upgraded = match browser_upgrade.await {
        Ok(io) => io,
        Err(error) => {
            error!(error = %error, "Browser WebSocket upgrade IO failed");
            return;
        },
    };

    let mut vite_io = TokioIo::new(vite_upgraded);
    let mut browser_io = TokioIo::new(browser_upgraded);

    match tokio::io::copy_bidirectional(&mut browser_io, &mut vite_io).await {
        Ok((browser_to_vite, vite_to_browser)) => {
            debug!(
                browser_to_vite,
                vite_to_browser, "WebSocket proxy connection closed"
            );
        },
        Err(error) => {
            debug!(error = %error, "WebSocket proxy pipe ended");
        },
    }
}
