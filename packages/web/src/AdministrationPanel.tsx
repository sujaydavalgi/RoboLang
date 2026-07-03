import { useCallback, useEffect, useState } from "react";
import type { RbacAction } from "./controlCenterRbac";

type ApiKeyRow = {
  key_id: string;
  role: string;
  label?: string;
  tenant_id?: string;
  source?: string;
};

type UserRow = {
  user_id: string;
  display_name: string;
  email?: string;
  role: string;
  api_key_id?: string;
  enabled: boolean;
};

type ScheduleRow = {
  id: string;
  profile: string;
  format: string;
  destination_url: string;
  interval_hours: number;
  enabled: boolean;
  last_status?: string;
};

type TwinSummary = {
  twin_id: string;
  program: string;
  readiness_score: number;
  mission_ready: boolean;
  history_count?: number;
};

type Props = {
  baseUrl: string;
  authHeaders: () => HeadersInit;
  can: (action: RbacAction) => boolean;
  hasToken: boolean;
};

const ROLES = [
  "administrator",
  "supervisor",
  "developer",
  "operator",
  "safety_officer",
  "auditor",
];

const CHANNEL_TYPES = ["webhook", "email", "pagerduty", "teams", "log"] as const;

export function AdministrationPanel({ baseUrl, authHeaders, can, hasToken }: Props) {
  const [keys, setKeys] = useState<ApiKeyRow[]>([]);
  const [users, setUsers] = useState<UserRow[]>([]);
  const [secrets, setSecrets] = useState<Record<string, unknown>[]>([]);
  const [schedules, setSchedules] = useState<ScheduleRow[]>([]);
  const [integrations, setIntegrations] = useState<Record<string, unknown> | null>(null);
  const [twins, setTwins] = useState<TwinSummary[]>([]);
  const [alertChannelsJson, setAlertChannelsJson] = useState("[]");
  const [persistPath, setPersistPath] = useState("");
  const [usersPath, setUsersPath] = useState("");
  const [newRole, setNewRole] = useState("operator");
  const [newLabel, setNewLabel] = useState("");
  const [newUserId, setNewUserId] = useState("");
  const [newUserName, setNewUserName] = useState("");
  const [newUserEmail, setNewUserEmail] = useState("");
  const [newUserRole, setNewUserRole] = useState("operator");
  const [channelType, setChannelType] = useState<(typeof CHANNEL_TYPES)[number]>("webhook");
  const [channelUrl, setChannelUrl] = useState("");
  const [channelEmail, setChannelEmail] = useState("");
  const [channelRoutingKey, setChannelRoutingKey] = useState("spanda");
  const [createdToken, setCreatedToken] = useState<string | null>(null);
  const [scheduleProfile, setScheduleProfile] = useState("defense");
  const [scheduleUrl, setScheduleUrl] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    if (!hasToken) return;
    setBusy(true);
    setError(null);
    try {
      const headers = authHeaders();
      const [keysRes, usersRes, secretsRes, schedulesRes, integrationsRes, channelsRes, twinsRes] =
        await Promise.all([
          fetch(`${baseUrl}/v1/admin/api-keys`, { headers }),
          fetch(`${baseUrl}/v1/admin/users`, { headers }),
          can("Deploy") ? fetch(`${baseUrl}/v1/secrets`, { headers }) : Promise.resolve(null),
          fetch(`${baseUrl}/v1/reports/schedules`),
          can("Deploy")
            ? fetch(`${baseUrl}/v1/admin/integrations`, { headers })
            : Promise.resolve(null),
          fetch(`${baseUrl}/v1/admin/alert-channels`, { headers }),
          fetch(`${baseUrl}/v1/twins`, { headers }),
        ]);
      if (keysRes.ok) {
        const body = await keysRes.json();
        setKeys(body.keys ?? []);
        setPersistPath(body.persist_path ?? "");
      }
      if (usersRes.ok) {
        const body = await usersRes.json();
        setUsers(body.users ?? []);
        setUsersPath(body.persist_path ?? "");
      }
      if (secretsRes?.ok) {
        const body = await secretsRes.json();
        setSecrets(body.secrets ?? []);
      }
      const schedulesBody = await (schedulesRes.ok ? schedulesRes.json() : { schedules: [] });
      setSchedules(schedulesBody.schedules ?? []);
      if (integrationsRes?.ok) {
        setIntegrations(await integrationsRes.json());
      }
      if (channelsRes?.ok) {
        const body = await channelsRes.json();
        setAlertChannelsJson(JSON.stringify(body.channels ?? [], null, 2));
      }
      if (twinsRes.ok) {
        const body = await twinsRes.json();
        setTwins(body.twins ?? []);
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }, [authHeaders, baseUrl, can, hasToken]);

  const syncTwinCloud = async () => {
    if (!can("Operate")) return;
    setBusy(true);
    try {
      const res = await fetch(`${baseUrl}/v1/twins/sync`, {
        method: "POST",
        headers: { "Content-Type": "application/json", ...authHeaders() },
        body: "{}",
      });
      if (!res.ok) throw new Error(`sync twin ${res.status}`);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  useEffect(() => {
    void load();
  }, [load]);

  const createKey = async () => {
    setBusy(true);
    setCreatedToken(null);
    try {
      const res = await fetch(`${baseUrl}/v1/admin/api-keys`, {
        method: "POST",
        headers: authHeaders(),
        body: JSON.stringify({ role: newRole, label: newLabel || undefined }),
      });
      if (!res.ok) throw new Error(`create key ${res.status}`);
      const body = await res.json();
      setCreatedToken(body.token ?? null);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const revokeKey = async (keyId: string) => {
    if (keyId === "env-default") return;
    setBusy(true);
    try {
      const res = await fetch(`${baseUrl}/v1/admin/api-keys/${encodeURIComponent(keyId)}`, {
        method: "DELETE",
        headers: authHeaders(),
      });
      if (!res.ok) throw new Error(`revoke ${res.status}`);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const patchKeyRole = async (keyId: string, role: string) => {
    if (keyId === "env-default") return;
    setBusy(true);
    try {
      const res = await fetch(`${baseUrl}/v1/admin/api-keys/${encodeURIComponent(keyId)}`, {
        method: "PATCH",
        headers: authHeaders(),
        body: JSON.stringify({ role }),
      });
      if (!res.ok) throw new Error(`patch ${res.status}`);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const createUser = async () => {
    if (!newUserId.trim() || !newUserName.trim()) return;
    setBusy(true);
    try {
      const res = await fetch(`${baseUrl}/v1/admin/users`, {
        method: "POST",
        headers: authHeaders(),
        body: JSON.stringify({
          user_id: newUserId,
          display_name: newUserName,
          email: newUserEmail || undefined,
          role: newUserRole,
        }),
      });
      if (!res.ok) throw new Error(`create user ${res.status}`);
      setNewUserId("");
      setNewUserName("");
      setNewUserEmail("");
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const toggleUser = async (userId: string, enabled: boolean) => {
    setBusy(true);
    try {
      const res = await fetch(`${baseUrl}/v1/admin/users/${encodeURIComponent(userId)}`, {
        method: "PATCH",
        headers: authHeaders(),
        body: JSON.stringify({ enabled }),
      });
      if (!res.ok) throw new Error(`patch user ${res.status}`);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const deleteUser = async (userId: string) => {
    setBusy(true);
    try {
      const res = await fetch(`${baseUrl}/v1/admin/users/${encodeURIComponent(userId)}`, {
        method: "DELETE",
        headers: authHeaders(),
      });
      if (!res.ok) throw new Error(`delete user ${res.status}`);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const addChannelToJson = () => {
    let channels: unknown[] = [];
    try {
      channels = JSON.parse(alertChannelsJson) as unknown[];
    } catch {
      channels = [];
    }
    let entry: Record<string, unknown>;
    if (channelType === "log") {
      entry = { log: null };
    } else if (channelType === "email") {
      entry = { email: { to: channelEmail } };
    } else if (channelType === "pagerduty") {
      entry = {
        pagerduty: { url: channelUrl, routing_key: channelRoutingKey },
      };
    } else if (channelType === "teams") {
      entry = { teams: { url: channelUrl } };
    } else {
      entry = { webhook: { url: channelUrl } };
    }
    channels.push(entry);
    setAlertChannelsJson(JSON.stringify(channels, null, 2));
  };

  const saveAlertChannels = async () => {
    setBusy(true);
    try {
      const channels = JSON.parse(alertChannelsJson) as unknown[];
      const res = await fetch(`${baseUrl}/v1/admin/alert-channels`, {
        method: "PUT",
        headers: authHeaders(),
        body: JSON.stringify({ channels, use_env_fallback: false }),
      });
      if (!res.ok) throw new Error(`alert channels ${res.status}`);
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const createSchedule = async () => {
    if (!can("Deploy") || !scheduleUrl.trim()) return;
    setBusy(true);
    try {
      const res = await fetch(`${baseUrl}/v1/reports/schedules`, {
        method: "POST",
        headers: authHeaders(),
        body: JSON.stringify({
          profile: scheduleProfile,
          destination_url: scheduleUrl,
          format: "markdown",
          interval_hours: 24,
        }),
      });
      if (!res.ok) throw new Error(`schedule ${res.status}`);
      setScheduleUrl("");
      await load();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  if (!hasToken) {
    return <p className="demo-hint">Sign in as administrator to manage users and integrations.</p>;
  }

  if (!can("Delete")) {
    return (
      <p className="demo-hint">
        Administration requires the <strong>administrator</strong> role.
      </p>
    );
  }

  return (
    <section className="cc-admin-panel">
      <header className="cc-section-header">
        <h3>Administration</h3>
        <button type="button" onClick={() => void load()} disabled={busy}>
          Refresh
        </button>
      </header>
      {error && <p className="error">{error}</p>}

      <h4>User directory</h4>
      <p className="demo-hint">Persist path: <code>{usersPath || "—"}</code></p>
      <table>
        <thead>
          <tr>
            <th>User ID</th>
            <th>Name</th>
            <th>Email</th>
            <th>Role</th>
            <th>API key</th>
            <th>Enabled</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {users.map((user) => (
            <tr key={user.user_id}>
              <td><code>{user.user_id}</code></td>
              <td>{user.display_name}</td>
              <td>{user.email ?? "—"}</td>
              <td>{user.role}</td>
              <td>{user.api_key_id ?? "—"}</td>
              <td>{user.enabled ? "yes" : "no"}</td>
              <td className="cc-action-bar">
                <button type="button" onClick={() => void toggleUser(user.user_id, !user.enabled)} disabled={busy}>
                  {user.enabled ? "Disable" : "Enable"}
                </button>
                <button type="button" onClick={() => void deleteUser(user.user_id)} disabled={busy}>
                  Delete
                </button>
              </td>
            </tr>
          ))}
          {users.length === 0 && (
            <tr>
              <td colSpan={7}>No users — add one or load a config with human operators.</td>
            </tr>
          )}
        </tbody>
      </table>
      <div className="digital-thread-filters">
        <label>User ID<input value={newUserId} onChange={(e) => setNewUserId(e.target.value)} /></label>
        <label>Display name<input value={newUserName} onChange={(e) => setNewUserName(e.target.value)} /></label>
        <label>Email<input value={newUserEmail} onChange={(e) => setNewUserEmail(e.target.value)} /></label>
        <label>
          Role
          <select value={newUserRole} onChange={(e) => setNewUserRole(e.target.value)}>
            {ROLES.map((role) => (
              <option key={role} value={role}>{role}</option>
            ))}
          </select>
        </label>
        <button type="button" onClick={() => void createUser()} disabled={busy}>Add user</button>
      </div>

      <h4>API keys</h4>
      <p className="demo-hint">Persist path: <code>{persistPath || "—"}</code></p>
      <table>
        <thead>
          <tr>
            <th>Key ID</th>
            <th>Role</th>
            <th>Label</th>
            <th>Source</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {keys.map((key) => (
            <tr key={key.key_id}>
              <td><code>{key.key_id}</code></td>
              <td>
                {key.key_id === "env-default" ? (
                  key.role
                ) : (
                  <select
                    value={String(key.role).replace(/^Role::/, "").toLowerCase()}
                    onChange={(event) => void patchKeyRole(key.key_id, event.target.value)}
                  >
                    {ROLES.map((role) => (
                      <option key={role} value={role}>{role}</option>
                    ))}
                  </select>
                )}
              </td>
              <td>{key.label ?? "—"}</td>
              <td>{key.source ?? "file"}</td>
              <td>
                {key.key_id !== "env-default" && (
                  <button type="button" onClick={() => void revokeKey(key.key_id)} disabled={busy}>
                    Revoke
                  </button>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <div className="digital-thread-filters">
        <label>
          Role
          <select value={newRole} onChange={(event) => setNewRole(event.target.value)}>
            {ROLES.map((role) => (
              <option key={role} value={role}>{role}</option>
            ))}
          </select>
        </label>
        <label>Label<input value={newLabel} onChange={(event) => setNewLabel(event.target.value)} /></label>
        <button type="button" onClick={() => void createKey()} disabled={busy}>Create API key</button>
      </div>
      {createdToken && (
        <p className="cc-created-token">New token (copy now): <code>{createdToken}</code></p>
      )}

      <h4>Alert channels</h4>
      <div className="digital-thread-filters">
        <label>
          Type
          <select value={channelType} onChange={(e) => setChannelType(e.target.value as typeof channelType)}>
            {CHANNEL_TYPES.map((t) => (
              <option key={t} value={t}>{t}</option>
            ))}
          </select>
        </label>
        {(channelType === "webhook" || channelType === "pagerduty" || channelType === "teams") && (
          <label>URL<input value={channelUrl} onChange={(e) => setChannelUrl(e.target.value)} /></label>
        )}
        {channelType === "email" && (
          <label>To<input value={channelEmail} onChange={(e) => setChannelEmail(e.target.value)} /></label>
        )}
        {channelType === "pagerduty" && (
          <label>Routing key<input value={channelRoutingKey} onChange={(e) => setChannelRoutingKey(e.target.value)} /></label>
        )}
        <button type="button" onClick={addChannelToJson}>Add to list</button>
        <button type="button" onClick={() => void saveAlertChannels()} disabled={busy}>Save channels</button>
      </div>
      <textarea
        className="cc-channels-json"
        rows={8}
        value={alertChannelsJson}
        onChange={(e) => setAlertChannelsJson(e.target.value)}
      />

      <h4>Secrets metadata</h4>
      {can("Deploy") ? (
        secrets.length > 0 ? (
          <pre>{JSON.stringify(secrets, null, 2)}</pre>
        ) : (
          <p className="demo-hint">No secret metadata registered.</p>
        )
      ) : (
        <p className="demo-hint">Deploy permission required to list secrets.</p>
      )}

      <h4>Report schedules</h4>
      <ul>
        {schedules.map((schedule) => (
          <li key={schedule.id}>
            <code>{schedule.id}</code> — {schedule.profile} → {schedule.destination_url} (
            {schedule.interval_hours}h) {schedule.last_status ?? ""}
          </li>
        ))}
        {schedules.length === 0 && <li>No schedules</li>}
      </ul>
      {can("Deploy") && (
        <div className="digital-thread-filters">
          <label>Profile<input value={scheduleProfile} onChange={(e) => setScheduleProfile(e.target.value)} /></label>
          <label>Webhook URL<input value={scheduleUrl} onChange={(e) => setScheduleUrl(e.target.value)} placeholder="https://hooks.example.com/reports" /></label>
          <button type="button" onClick={() => void createSchedule()} disabled={busy}>Add schedule</button>
        </div>
      )}

      <h4>Twin Cloud registry</h4>
      <p className="demo-hint">
        Mission twin snapshots from edge push or sync. Mutations require Operate permission.
      </p>
      <div className="digital-thread-filters">
        <button type="button" onClick={() => void load()} disabled={busy}>
          Refresh twins
        </button>
        {can("Operate") && (
          <button type="button" onClick={() => void syncTwinCloud()} disabled={busy}>
            Sync loaded program
          </button>
        )}
      </div>
      <table>
        <thead>
          <tr>
            <th>Twin</th>
            <th>Program</th>
            <th>Readiness</th>
            <th>Mission ready</th>
            <th>History</th>
          </tr>
        </thead>
        <tbody>
          {twins.map((twin) => (
            <tr key={twin.twin_id}>
              <td><code>{twin.twin_id}</code></td>
              <td>{twin.program}</td>
              <td>{twin.readiness_score}</td>
              <td>{twin.mission_ready ? "yes" : "no"}</td>
              <td>{twin.history_count ?? 1}</td>
            </tr>
          ))}
          {twins.length === 0 && (
            <tr>
              <td colSpan={5}>No twins registered — use Twin Cloud tab or CLI push.</td>
            </tr>
          )}
        </tbody>
      </table>

      <h4>Integrations summary</h4>
      {integrations ? <pre>{JSON.stringify(integrations, null, 2)}</pre> : <p className="demo-hint">Loading…</p>}
    </section>
  );
}
