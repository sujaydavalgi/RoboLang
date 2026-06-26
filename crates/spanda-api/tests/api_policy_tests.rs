//! Control Center API versioning and rate-limit policy tests.

use spanda_api::handlers::handle_request;
use spanda_api::state::ControlCenterState;
use spanda_deploy_http::HttpRequest;
use spanda_security::RateLimiter;

#[test]
fn unsupported_api_version_header_is_rejected() {
    let mut state = ControlCenterState::new();
    let (response, _) = handle_request(
        &mut state,
        &HttpRequest {
            method: "GET".into(),
            path: "/v1/dashboard".into(),
            body: String::new(),
            authorization: None,
        },
        "GET /v1/dashboard HTTP/1.1\r\nX-Spanda-Api-Version: v2\r\n\r\n",
    );
    assert_eq!(response.status, 400);
    assert!(response.body.contains("unsupported api version"));
}

#[test]
fn rate_limit_returns_429_when_exceeded() {
    let mut state = ControlCenterState::new();
    state.rate_limiter = RateLimiter::with_limit(1);
    let raw = "GET /v1/dashboard HTTP/1.1\r\n\r\n";
    let request = HttpRequest {
        method: "GET".into(),
        path: "/v1/dashboard".into(),
        body: String::new(),
        authorization: None,
    };
    let (first, _) = handle_request(&mut state, &request, raw);
    assert_eq!(first.status, 200);
    let (second, _) = handle_request(&mut state, &request, raw);
    assert_eq!(second.status, 429);
    assert!(second.body.contains("rate limit exceeded"));
}

#[test]
fn version_endpoint_documents_policy() {
    let mut state = ControlCenterState::new();
    let (response, _) = handle_request(
        &mut state,
        &HttpRequest {
            method: "GET".into(),
            path: "/v1/version".into(),
            body: String::new(),
            authorization: None,
        },
        "",
    );
    assert_eq!(response.status, 200);
    assert!(response.body.contains("supported_versions"));
    assert!(response.body.contains("/v2/"));
}
