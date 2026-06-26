//! OpenAPI 3.1 spec must document every REST v1 route in `openapi_routes::REST_V1_ROUTES`.
use spanda_api::openapi_routes::REST_V1_ROUTES;

#[test]
fn openapi_documents_all_rest_v1_routes() {
    let spec: serde_json::Value =
        serde_json::from_str(include_str!("../src/static/openapi.json")).expect("parse openapi");
    let paths = spec["paths"]
        .as_object()
        .expect("openapi paths object");

    for route in REST_V1_ROUTES {
        let path_entry = paths
            .get(route.path)
            .unwrap_or_else(|| panic!("missing openapi path {}", route.path));
        let method = route.method.to_ascii_lowercase();
        assert!(
            path_entry.get(&method).is_some(),
            "missing openapi method {method} for {}",
            route.path
        );
    }
}

#[test]
fn openapi_has_no_stale_rest_paths() {
    let spec: serde_json::Value =
        serde_json::from_str(include_str!("../src/static/openapi.json")).expect("parse openapi");
    let paths = spec["paths"]
        .as_object()
        .expect("openapi paths object");

    let documented: std::collections::HashSet<(&str, &str)> = paths
        .iter()
        .flat_map(|(path, methods)| {
            methods
                .as_object()
                .into_iter()
                .flatten()
                .filter_map(move |(method, _)| {
                    if method == "parameters" {
                        return None;
                    }
                    Some((method.as_str(), path.as_str()))
                })
        })
        .collect();

    let expected: std::collections::HashSet<(String, String)> = REST_V1_ROUTES
        .iter()
        .map(|route| (route.method.to_ascii_lowercase(), route.path.to_string()))
        .collect();

    for (method, path) in documented {
        if path == "/v1/rpc" {
            continue;
        }
        assert!(
            expected.contains(&(method.to_string(), path.to_string())),
            "stale openapi entry {method} {path} not in REST_V1_ROUTES"
        );
    }
}
