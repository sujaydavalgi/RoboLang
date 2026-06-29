/**
 * Unified Entity Graph panel for Control Center — browse, search, and inspect entities.
 * @module
 */
import { useMemo, useState } from "react";

export type EntitySummary = {
  id: string;
  kind?: string;
  entity_type?: string;
  display_name?: string;
  health_status?: string;
  readiness_status?: string;
  trust_status?: string;
  lifecycle_state?: string;
  parent_id?: string;
  capabilities?: string[];
  tags?: string[];
};

export type EntityRelationship = {
  from_id: string;
  to_id: string;
  kind: string;
  label?: string;
};

export type EntityGraphPayload = {
  nodes: EntitySummary[];
  edges: EntityRelationship[];
};

export type RegisterEntityInput = {
  id: string;
  entity_type: string;
  display_name?: string;
  parent_id?: string;
  capabilities?: string[];
};

export type EntityWriteActions = {
  canWrite: boolean;
  busy?: boolean;
  onRegister: (input: RegisterEntityInput) => Promise<void>;
  onTag: (entityId: string, tags: string[]) => Promise<void>;
  onRelate: (fromId: string, toId: string, kind: string, label?: string) => Promise<void>;
  onSync: () => Promise<void>;
};

type Props = {
  entities: EntitySummary[];
  graph: EntityGraphPayload | null;
  selectedId: string | null;
  onSelect: (id: string | null) => void;
  kindFilter: string;
  onKindFilterChange: (value: string) => void;
  search: string;
  onSearchChange: (value: string) => void;
  detail: Record<string, unknown> | null;
  relationships: EntityRelationship[];
  loading?: boolean;
  write?: EntityWriteActions;
};

const HEALTH_COLORS: Record<string, string> = {
  healthy: "#22c55e",
  warning: "#eab308",
  degraded: "#f97316",
  offline: "#94a3b8",
  critical: "#ef4444",
  unknown: "#64748b",
};

const KIND_ORDER = [
  "organization",
  "fleet",
  "robot",
  "human",
  "wearable",
  "compute",
  "device",
  "sensor",
  "camera",
  "provider",
  "package",
  "control_center",
  "hazard",
  "digital_twin",
];

function kindRank(kind: string): number {
  const idx = KIND_ORDER.indexOf(kind);
  return idx >= 0 ? idx : 99;
}

export function EntityGraphPanel({
  entities,
  graph,
  selectedId,
  onSelect,
  kindFilter,
  onKindFilterChange,
  search,
  onSearchChange,
  detail,
  relationships,
  loading,
  write,
}: Props) {
  const [view, setView] = useState<"list" | "graph">("list");
  const [writeOpen, setWriteOpen] = useState(false);
  const [regId, setRegId] = useState("");
  const [regType, setRegType] = useState("calibration_station");
  const [regName, setRegName] = useState("");
  const [regParent, setRegParent] = useState("");
  const [tagInput, setTagInput] = useState("");
  const [relFrom, setRelFrom] = useState("");
  const [relTo, setRelTo] = useState("");
  const [relKind, setRelKind] = useState("depends_on");
  const [relLabel, setRelLabel] = useState("");

  const kinds = useMemo(() => {
    const set = new Set<string>();
    for (const entity of entities) {
      if (entity.kind) set.add(entity.kind);
    }
    return Array.from(set).sort();
  }, [entities]);

  const filtered = useMemo(() => {
    const needle = search.trim().toLowerCase();
    return entities
      .filter((e) => {
        if (kindFilter && e.kind !== kindFilter) return false;
        if (!needle) return true;
        const hay = `${e.id} ${e.display_name ?? ""} ${e.kind ?? ""}`.toLowerCase();
        return hay.includes(needle);
      })
      .sort((a, b) => {
        const ka = kindRank(a.kind ?? "");
        const kb = kindRank(b.kind ?? "");
        if (ka !== kb) return ka - kb;
        return a.id.localeCompare(b.id);
      });
  }, [entities, kindFilter, search]);

  const miniGraph = useMemo(() => {
    if (!graph || graph.nodes.length === 0) return null;
    const focus = selectedId ?? filtered[0]?.id;
    if (!focus) return null;
    const related = new Set<string>([focus]);
    for (const edge of graph.edges) {
      if (edge.from_id === focus) related.add(edge.to_id);
      if (edge.to_id === focus) related.add(edge.from_id);
    }
    const nodes = graph.nodes.filter((n) => related.has(n.id)).slice(0, 12);
    const nodeIds = new Set(nodes.map((n) => n.id));
    const edges = graph.edges.filter(
      (e) => nodeIds.has(e.from_id) && nodeIds.has(e.to_id),
    );
    return { nodes, edges, focus };
  }, [graph, selectedId, filtered]);

  const submitRegister = async () => {
    if (!write?.canWrite || !regId.trim() || !regType.trim()) return;
    await write.onRegister({
      id: regId.trim(),
      entity_type: regType.trim(),
      display_name: regName.trim() || undefined,
      parent_id: regParent.trim() || undefined,
    });
    setRegId("");
    setRegName("");
    setRegParent("");
  };

  const submitTag = async () => {
    if (!write?.canWrite || !selectedId) return;
    const tags = tagInput
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);
    if (tags.length === 0) return;
    await write.onTag(selectedId, tags);
    setTagInput("");
  };

  const submitRelate = async () => {
    const from = (relFrom.trim() || selectedId || "").trim();
    if (!write?.canWrite || !from || !relTo.trim() || !relKind.trim()) return;
    await write.onRelate(from, relTo.trim(), relKind.trim(), relLabel.trim() || undefined);
    setRelLabel("");
  };

  const submitSync = async () => {
    if (!write?.canWrite) return;
    await write.onSync();
  };

  return (
    <div className="entity-graph-panel">
      {write && (
        <div className="entity-write-panel">
          <button type="button" onClick={() => setWriteOpen((v) => !v)}>
            {writeOpen ? "Hide" : "Show"} entity mutations
          </button>
          {!write.canWrite && (
            <p className="demo-hint">
              Set <code>VITE_SPANDA_API_KEY</code> to register, tag, relate, or sync entities.
            </p>
          )}
          {writeOpen && write.canWrite && (
            <div className="entity-write-forms">
              <fieldset disabled={write.busy}>
                <legend>Register entity</legend>
                <div className="entity-write-row">
                  <input placeholder="id" value={regId} onChange={(e) => setRegId(e.target.value)} />
                  <input
                    placeholder="entity_type"
                    value={regType}
                    onChange={(e) => setRegType(e.target.value)}
                  />
                  <input
                    placeholder="display_name"
                    value={regName}
                    onChange={(e) => setRegName(e.target.value)}
                  />
                  <input
                    placeholder="parent_id"
                    value={regParent}
                    onChange={(e) => setRegParent(e.target.value)}
                  />
                  <button type="button" className="primary" onClick={() => void submitRegister()}>
                    Register
                  </button>
                </div>
              </fieldset>
              <fieldset disabled={write.busy || !selectedId}>
                <legend>Tag selected ({selectedId ?? "none"})</legend>
                <div className="entity-write-row">
                  <input
                    placeholder="comma-separated tags"
                    value={tagInput}
                    onChange={(e) => setTagInput(e.target.value)}
                  />
                  <button type="button" onClick={() => void submitTag()} disabled={!selectedId}>
                    Add tags
                  </button>
                </div>
              </fieldset>
              <fieldset disabled={write.busy}>
                <legend>Relate entities</legend>
                <div className="entity-write-row">
                  <input
                    placeholder="from_id"
                    value={relFrom || selectedId || ""}
                    onChange={(e) => setRelFrom(e.target.value)}
                  />
                  <input placeholder="to_id" value={relTo} onChange={(e) => setRelTo(e.target.value)} />
                  <input
                    placeholder="kind"
                    value={relKind}
                    onChange={(e) => setRelKind(e.target.value)}
                  />
                  <input
                    placeholder="label (optional)"
                    value={relLabel}
                    onChange={(e) => setRelLabel(e.target.value)}
                  />
                  <button type="button" onClick={() => void submitRelate()}>
                    Relate
                  </button>
                </div>
              </fieldset>
              <button type="button" className="primary" onClick={() => void submitSync()}>
                Sync overlay to TOML
              </button>
            </div>
          )}
        </div>
      )}
      <div className="entity-graph-toolbar">
        <input
          type="search"
          placeholder="Search entities…"
          value={search}
          onChange={(e) => onSearchChange(e.target.value)}
        />
        <select
          value={kindFilter}
          onChange={(e) => onKindFilterChange(e.target.value)}
          aria-label="Filter by entity kind"
        >
          <option value="">All kinds</option>
          {kinds.map((k) => (
            <option key={k} value={k}>
              {k}
            </option>
          ))}
        </select>
        <button type="button" onClick={() => setView("list")} disabled={view === "list"}>
          List
        </button>
        <button type="button" onClick={() => setView("graph")} disabled={view === "graph"}>
          Graph
        </button>
        {loading && <span className="demo-hint">Loading…</span>}
      </div>

      <div className="entity-graph-layout">
        <div className="entity-list">
          <p className="demo-hint">{filtered.length} entities</p>
          <ul>
            {filtered.map((entity) => {
              const health = entity.health_status ?? "unknown";
              return (
                <li key={entity.id}>
                  <button
                    type="button"
                    className={selectedId === entity.id ? "selected" : ""}
                    onClick={() => onSelect(entity.id)}
                  >
                    <span
                      className="entity-health-dot"
                      style={{ background: HEALTH_COLORS[health] ?? HEALTH_COLORS.unknown }}
                    />
                    <strong>{entity.display_name ?? entity.id}</strong>
                    <span className="entity-kind">{entity.kind ?? "entity"}</span>
                  </button>
                </li>
              );
            })}
          </ul>
        </div>

        <div className="entity-detail">
          {!selectedId && <p>Select an entity to inspect health, readiness, trust, and relationships.</p>}
          {selectedId && detail && (
            <>
              <h3>{String(detail.display_name ?? detail.id ?? selectedId)}</h3>
              <dl className="entity-meta">
                <dt>ID</dt>
                <dd>{String(detail.id ?? selectedId)}</dd>
                <dt>Kind</dt>
                <dd>{String(detail.entity_type ?? detail.kind ?? "—")}</dd>
                <dt>Health</dt>
                <dd>{String(detail.health_status ?? "—")}</dd>
                <dt>Readiness</dt>
                <dd>{String(detail.readiness_status ?? "—")}</dd>
                <dt>Trust</dt>
                <dd>{String(detail.trust_status ?? "—")}</dd>
                <dt>Lifecycle</dt>
                <dd>{String(detail.lifecycle_state ?? "—")}</dd>
                {Array.isArray(detail.capabilities) && detail.capabilities.length > 0 && (
                  <>
                    <dt>Capabilities</dt>
                    <dd>{(detail.capabilities as string[]).join(", ")}</dd>
                  </>
                )}
              </dl>
              {relationships.length > 0 && (
                <>
                  <h4>Relationships</h4>
                  <table>
                    <thead>
                      <tr>
                        <th>Direction</th>
                        <th>Kind</th>
                        <th>Peer</th>
                      </tr>
                    </thead>
                    <tbody>
                      {relationships.map((rel, idx) => {
                        const outbound = rel.from_id === selectedId;
                        const peer = outbound ? rel.to_id : rel.from_id;
                        return (
                          <tr key={`${rel.from_id}-${rel.to_id}-${idx}`}>
                            <td>{outbound ? "→" : "←"}</td>
                            <td>{rel.kind}</td>
                            <td>
                              <button type="button" onClick={() => onSelect(peer)}>
                                {peer}
                              </button>
                            </td>
                          </tr>
                        );
                      })}
                    </tbody>
                  </table>
                </>
              )}
            </>
          )}

          {view === "graph" && miniGraph && (
            <div className="entity-mini-graph">
              <h4>Neighborhood graph</h4>
              <svg viewBox="0 0 480 220" role="img" aria-label="Entity neighborhood graph">
                {miniGraph.edges.map((edge, idx) => {
                  const fromIdx = miniGraph.nodes.findIndex((n) => n.id === edge.from_id);
                  const toIdx = miniGraph.nodes.findIndex((n) => n.id === edge.to_id);
                  if (fromIdx < 0 || toIdx < 0) return null;
                  const x1 = 40 + (fromIdx % 4) * 110;
                  const y1 = 30 + Math.floor(fromIdx / 4) * 70;
                  const x2 = 40 + (toIdx % 4) * 110;
                  const y2 = 30 + Math.floor(toIdx / 4) * 70;
                  return (
                    <line
                      key={`${edge.from_id}-${edge.to_id}-${idx}`}
                      x1={x1}
                      y1={y1}
                      x2={x2}
                      y2={y2}
                      stroke="#64748b"
                      strokeWidth={1}
                    />
                  );
                })}
                {miniGraph.nodes.map((node, idx) => {
                  const x = 40 + (idx % 4) * 110;
                  const y = 30 + Math.floor(idx / 4) * 70;
                  const selected = node.id === miniGraph.focus;
                  return (
                    <g key={node.id} onClick={() => onSelect(node.id)} style={{ cursor: "pointer" }}>
                      <circle
                        cx={x}
                        cy={y}
                        r={selected ? 14 : 10}
                        fill={selected ? "#6366f1" : "#334155"}
                      />
                      <text x={x} y={y + 24} textAnchor="middle" fontSize={9} fill="#e2e8f0">
                        {(node.display_name ?? node.id).slice(0, 12)}
                      </text>
                    </g>
                  );
                })}
              </svg>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
