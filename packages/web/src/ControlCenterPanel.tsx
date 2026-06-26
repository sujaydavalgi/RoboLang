import { useCallback, useEffect, useState } from "react";

type DashboardData = {
  device_pool: {
    total: number;
    healthy: number;
    degraded: number;
    discovered: number;
    failed: number;
  };
  fleet_agent_count: number;
  alert_count: number;
};

type FleetAgent = {
  robot_name: string;
  url: string;
  token?: string;
};

type DeviceEntry = {
  id: string;
  device_type: string;
  lifecycle_state: string;
  assigned_robot?: string;
};

type Tab = "dashboard" | "fleet" | "readiness";

type Props = {
  apiBase: string;
};

export function ControlCenterPanel({ apiBase }: Props) {
  const [tab, setTab] = useState<Tab>("dashboard");
  const [dashboard, setDashboard] = useState<DashboardData | null>(null);
  const [agents, setAgents] = useState<FleetAgent[]>([]);
  const [devices, setDevices] = useState<DeviceEntry[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const base = apiBase.replace(/\/$/, "");

  const load = useCallback(async () => {
    setBusy(true);
    setError(null);
    try {
      const dashRes = await fetch(`${base}/v1/dashboard`);
      if (!dashRes.ok) throw new Error(`dashboard ${dashRes.status}`);
      setDashboard(await dashRes.json());

      const fleetRes = await fetch(`${base}/v1/fleet/agents`);
      if (!fleetRes.ok) throw new Error(`fleet ${fleetRes.status}`);
      const fleetBody = await fleetRes.json();
      setAgents(fleetBody.agents ?? []);

      const devRes = await fetch(`${base}/v1/devices`);
      if (!devRes.ok) throw new Error(`devices ${devRes.status}`);
      const devBody = await devRes.json();
      setDevices(devBody.devices ?? []);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }, [base]);

  useEffect(() => {
    void load();
  }, [load]);

  const pool = dashboard?.device_pool;

  return (
    <div className="panel control-center">
      <h2>Control Center</h2>
      <p className="demo-hint">
        API: <code>{base}</code> — run <code>spanda control-center serve</code>
      </p>
      <div className="toolbar">
        {(["dashboard", "fleet", "readiness"] as Tab[]).map((name) => (
          <button
            key={name}
            type="button"
            className={tab === name ? "primary" : undefined}
            onClick={() => setTab(name)}
          >
            {name.charAt(0).toUpperCase() + name.slice(1)}
          </button>
        ))}
        <button type="button" onClick={() => void load()} disabled={busy}>
          Refresh
        </button>
      </div>
      {error && <div className="error">{error}</div>}
      {tab === "dashboard" && pool && (
        <dl>
          <dt>Devices</dt>
          <dd>{pool.total}</dd>
          <dt>Healthy</dt>
          <dd>{pool.healthy}</dd>
          <dt>Fleet agents</dt>
          <dd>{dashboard?.fleet_agent_count ?? 0}</dd>
          <dt>Alerts</dt>
          <dd>{dashboard?.alert_count ?? 0}</dd>
        </dl>
      )}
      {tab === "fleet" && (
        <ul>
          {agents.length === 0 && <li>No fleet agents registered</li>}
          {agents.map((a) => (
            <li key={a.robot_name}>
              <strong>{a.robot_name}</strong> — {a.url}
            </li>
          ))}
        </ul>
      )}
      {tab === "readiness" && pool && (
        <dl>
          <dt>Mission-capable (healthy + assigned + verified)</dt>
          <dd>{pool.healthy}</dd>
          <dt>Degraded</dt>
          <dd>{pool.degraded}</dd>
          <dt>Discovered (unprovisioned)</dt>
          <dd>{pool.discovered}</dd>
          <dt>Device pool entries</dt>
          <dd>
            <ul>
              {devices.map((d) => (
                <li key={d.id}>
                  {d.id} ({d.lifecycle_state})
                </li>
              ))}
            </ul>
          </dd>
        </dl>
      )}
    </div>
  );
}
