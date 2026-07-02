"""Python SDK smoke tests."""

from urllib.parse import urlencode

from spanda_sdk import SpandaClient
from spanda_sdk.errors import SpandaError


def test_local_client_constructs():
    client = SpandaClient.local()
    assert "127.0.0.1" in client.base_url


def test_program_body_shape():
    client = SpandaClient.local()
    body = client._program_body("rover.sd")
    assert body["file"] == "rover.sd"


def test_entity_traceability_path():
    client = SpandaClient.local()
    captured: dict[str, str] = {}

    def fake_request(method, path, body=None, auth=False):
        captured["path"] = path
        return {}

    client._request = fake_request  # type: ignore[method-assign]
    client.entity_traceability(entity_id="rover-001", capability="nav")
    assert captured["path"] == "/v1/entities/traceability?entity_id=rover-001&capability=nav"


def test_register_entity_uses_auth():
    client = SpandaClient.local()
    captured: dict[str, object] = {}

    def fake_request(method, path, body=None, auth=False):
        captured["method"] = method
        captured["path"] = path
        captured["auth"] = auth
        return {"id": "bay-1"}

    client._request = fake_request  # type: ignore[method-assign]
    client.register_entity({"id": "bay-1"})
    assert captured == {
        "method": "POST",
        "path": "/v1/entities/register",
        "auth": True,
    }


def test_analytics_what_if_path():
    client = SpandaClient.local()
    captured: dict[str, str] = {}

    def fake_request(method, path, body=None, auth=False):
        captured["path"] = path
        return {}

    client._request = fake_request  # type: ignore[method-assign]
    client.analytics_what_if(scenario="gps_failure", all_values=True)
    assert captured["path"] == "/v1/analytics/what-if?all=1&scenario=gps_failure"


def test_analytics_time_travel_path_encodes_timestamp():
    client = SpandaClient.local()
    captured: dict[str, str] = {}

    def fake_request(method, path, body=None, auth=False):
        captured["path"] = path
        return {}

    client._request = fake_request  # type: ignore[method-assign]
    client.analytics_time_travel(at="T+00:01", inspect="decisions")
    assert captured["path"] == (
        f"/v1/analytics/time-travel?{urlencode({'at': 'T+00:01', 'inspect': 'decisions'})}"
    )
