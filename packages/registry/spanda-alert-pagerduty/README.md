# spanda-alert-pagerduty

PagerDuty Events API v2 integration for Spanda Control Center alerting.

## Status

**Experimental** — outbound dispatch and bi-directional incident sync via `spanda-ops` + Control Center API.

## Outbound (Spanda → PagerDuty)

```bash
export SPANDA_ALERT_PAGERDUTY_URL="https://events.pagerduty.com/v2/enqueue"
export SPANDA_ALERT_PAGERDUTY_ROUTING_KEY="your-routing-key"
```

Critical alerts include `incident_id` in `custom_details` when an SRE incident is auto-opened.

## Inbound (PagerDuty → Spanda)

Configure a webhook extension or custom integration to call:

```http
POST /v1/integrations/pagerduty/webhook
X-Spanda-Webhook-Secret: <optional shared secret>
Authorization: Bearer <SPANDA_API_KEY>
Content-Type: application/json

{"event":"incident.acknowledged","incident_id":"incident-123","assignee":"oncall"}
```

Or match by alert dedup key:

```json
{"event":"incident.resolve","dedup_key":"alert-42"}
```

Set `SPANDA_PAGERDUTY_WEBHOOK_SECRET` to require the `X-Spanda-Webhook-Secret` header (recommended in production).

## Outbound incident sync

When operators ack or resolve via `POST /v1/sre/incidents/{id}/ack|resolve`, Spanda sends matching PagerDuty `acknowledge` / `resolve` events when outbound env vars are set.
