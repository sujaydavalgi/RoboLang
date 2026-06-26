//! Control Center remote CLI routes must map to documented REST v1 OpenAPI paths.

use spanda_api::openapi_routes::REST_V1_ROUTES;
use std::collections::HashSet;

const CLI_ROUTES: &[(&str, &str)] = &[
    ("GET", "/v1/dashboard"),
    ("GET", "/v1/drift"),
    ("POST", "/v1/drift/scan"),
    ("GET", "/v1/drift/scans"),
    ("GET", "/v1/sre/incidents"),
    ("POST", "/v1/sre/incidents"),
    ("POST", "/v1/sre/incidents/{id}/ack"),
    ("POST", "/v1/sre/incidents/{id}/resolve"),
    ("GET", "/v1/config/approvals"),
    ("POST", "/v1/config/approvals"),
    ("POST", "/v1/config/approvals/{id}/approve"),
    ("POST", "/v1/config/approvals/{id}/reject"),
    ("GET", "/v1/compliance/evidence"),
    ("GET", "/v1/sre/summary"),
    ("GET", "/v1/devices"),
    ("GET", "/v1/devices/{id}"),
    ("PATCH", "/v1/devices/{id}"),
    ("POST", "/v1/devices/{id}/assign"),
    ("POST", "/v1/devices/{id}/quarantine"),
    ("POST", "/v1/devices/{id}/provision"),
    ("POST", "/v1/devices/{id}/trust"),
    ("GET", "/v1/ota/status"),
    ("POST", "/v1/ota/plan"),
    ("POST", "/v1/ota/execute"),
    ("POST", "/v1/readiness/run"),
    ("GET", "/v1/compliance/export"),
    ("GET", "/v1/alerts"),
    ("POST", "/v1/alerts/test"),
    ("GET", "/v1/config/snapshots"),
    ("POST", "/v1/config/snapshots"),
    ("GET", "/v1/trust/package"),
    ("GET", "/v1/executive/scorecard"),
    ("GET", "/v1/digital-thread/query"),
    ("GET", "/v1/reports/export"),
    ("POST", "/v1/provision"),
    ("GET", "/v1/secrets"),
    ("GET", "/v1/audit/mutations"),
];

#[test]
fn control_center_cli_routes_documented_in_openapi_registry() {
    let registry: HashSet<(&str, &str)> = REST_V1_ROUTES
        .iter()
        .map(|route| (route.method, route.path))
        .collect();

    for (method, path) in CLI_ROUTES {
        assert!(
            registry.contains(&(method, path)),
            "CLI route {method} {path} missing from REST_V1_ROUTES"
        );
    }
}
