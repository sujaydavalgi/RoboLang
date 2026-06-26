"""HTTP client for Spanda Control Center REST API v1."""

from __future__ import annotations

import json
import os
import urllib.error
import urllib.request
import uuid
from typing import Any, Mapping, Optional


class ControlCenterClient:
    """REST v1 client with optional Bearer auth and correlation IDs."""

    def __init__(
        self,
        base_url: Optional[str] = None,
        api_key: Optional[str] = None,
        timeout: float = 30.0,
    ) -> None:
        resolved_url = base_url or os.environ.get(
            "SPANDA_CONTROL_CENTER_URL", "http://127.0.0.1:8080"
        )
        self.base_url = resolved_url.rstrip("/")
        self.api_key = api_key if api_key is not None else os.environ.get("SPANDA_API_KEY")
        self.timeout = timeout

    def _request(
        self,
        method: str,
        path: str,
        body: Optional[Mapping[str, Any]] = None,
        auth: bool = False,
        correlation_id: Optional[str] = None,
    ) -> Any:
        url = f"{self.base_url}{path}"
        headers = {"Accept": "application/json"}
        cid = correlation_id or f"py-sdk-{uuid.uuid4().hex[:12]}"
        headers["X-Correlation-ID"] = cid
        data = None
        if body is not None:
            headers["Content-Type"] = "application/json"
            data = json.dumps(body).encode("utf-8")
        if auth and self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"
        req = urllib.request.Request(url, data=data, headers=headers, method=method)
        try:
            with urllib.request.urlopen(req, timeout=self.timeout) as resp:
                payload = resp.read().decode("utf-8")
                if not payload:
                    return {}
                return json.loads(payload)
        except urllib.error.HTTPError as exc:
            detail = exc.read().decode("utf-8", errors="replace")
            raise RuntimeError(f"{method} {path} failed ({exc.code}): {detail}") from exc

    def health(self) -> Any:
        return self._request("GET", "/v1/health")

    def dashboard(self) -> Any:
        return self._request("GET", "/v1/dashboard")

    def drift(self, baseline_id: str) -> Any:
        return self._request("GET", f"/v1/drift?baseline_id={baseline_id}")

    def ota_plan(
        self,
        strategy: str,
        version: str,
        *,
        canary_percent: int = 10,
        dry_run: bool = True,
        assignments: Optional[list] = None,
    ) -> Any:
        return self._request(
            "POST",
            "/v1/ota/plan",
            {
                "strategy": strategy,
                "version": version,
                "canary_percent": canary_percent,
                "dry_run": dry_run,
                "assignments": assignments or [],
            },
            auth=True,
        )

    def trust_package(self, name: str, version: Optional[str] = None) -> Any:
        path = f"/v1/trust/package?name={name}"
        if version:
            path += f"&version={version}"
        return self._request("GET", path)

    def sre_summary(self) -> Any:
        return self._request("GET", "/v1/sre/summary")

    def list_incidents(self) -> Any:
        return self._request("GET", "/v1/sre/incidents")

    def create_incident(
        self,
        title: str,
        *,
        description: str = "",
        severity: str = "warning",
        source_alert_id: Optional[str] = None,
    ) -> Any:
        body: dict[str, Any] = {
            "title": title,
            "description": description,
            "severity": severity,
        }
        if source_alert_id:
            body["source_alert_id"] = source_alert_id
        return self._request("POST", "/v1/sre/incidents", body, auth=True)

    def ack_incident(self, incident_id: str, *, assignee: Optional[str] = None) -> Any:
        body: dict[str, Any] = {}
        if assignee:
            body["assignee"] = assignee
        return self._request(
            "POST",
            f"/v1/sre/incidents/{incident_id}/ack",
            body,
            auth=True,
        )

    def resolve_incident(self, incident_id: str) -> Any:
        return self._request(
            "POST",
            f"/v1/sre/incidents/{incident_id}/resolve",
            {},
            auth=True,
        )

    def rpc(self, method: str, params: Optional[Mapping[str, Any]] = None) -> Any:
        return self._request(
            "POST",
            "/v1/rpc",
            {"method": method, "params": params or {}},
        )
