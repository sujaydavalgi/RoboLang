/**
 * Interactive SVG graph for digital thread capability-to-device traces.
 * @module
 */
import { useMemo, useState } from "react";

export type DigitalThreadGraphNode = {
  id: string;
  label: string;
  kind: string;
};

export type DigitalThreadGraphEdge = {
  from: string;
  to: string;
  relation: string;
};

export type DigitalThreadDeviceLink = {
  device_id: string;
  device_type: string;
  assigned_robot?: string | null;
  lifecycle_state?: string | null;
  related_capabilities: string[];
};

export type DigitalThreadLifecycleRow = {
  node_id: string;
  phase: string;
};

type Props = {
  nodes: DigitalThreadGraphNode[];
  edges: DigitalThreadGraphEdge[];
  deviceLinks?: DigitalThreadDeviceLink[];
  lifecycleRows?: DigitalThreadLifecycleRow[];
  selectedId?: string | null;
  onSelectNode?: (id: string | null) => void;
};

const LIFECYCLE_COLUMN: Record<string, number> = {
  requirement: 0,
  design: 1,
  deploy: 2,
  operate: 3,
  retire: 4,
};

const LIFECYCLE_COLORS: Record<string, string> = {
  requirement: "#a78bfa",
  design: "#6366f1",
  deploy: "#3b82f6",
  operate: "#22c55e",
  retire: "#94a3b8",
};

const KIND_COLUMN: Record<string, number> = {
  mission: 0,
  robot: 1,
  capability: 2,
  hardware: 3,
  sensor: 3,
  actuator: 3,
  provider: 4,
  package: 5,
  safety: 6,
};

const KIND_COLORS: Record<string, string> = {
  mission: "#6366f1",
  robot: "#3b82f6",
  capability: "#eab308",
  hardware: "#94a3b8",
  sensor: "#22d3ee",
  actuator: "#f97316",
  provider: "#22c55e",
  package: "#a3e635",
  safety: "#f87171",
};

const NODE_W = 128;
const NODE_H = 36;
const COL_GAP = 168;
const ROW_GAP = 52;
const PAD = 28;

function layoutGraph(
  nodes: DigitalThreadGraphNode[],
  lifecycleRows: DigitalThreadLifecycleRow[],
) {
  const phaseByNode = new Map(lifecycleRows.map((row) => [row.node_id, row.phase]));
  const useLifecycle = lifecycleRows.length > 0;
  const columnRows = new Map<number, number>();
  const positions = new Map<string, { x: number; y: number }>();
  const sorted = [...nodes].sort((left, right) => {
    const leftCol = useLifecycle
      ? (LIFECYCLE_COLUMN[phaseByNode.get(left.id) ?? ""] ?? 2)
      : (KIND_COLUMN[left.kind] ?? 3);
    const rightCol = useLifecycle
      ? (LIFECYCLE_COLUMN[phaseByNode.get(right.id) ?? ""] ?? 2)
      : (KIND_COLUMN[right.kind] ?? 3);
    return leftCol - rightCol || left.label.localeCompare(right.label);
  });
  for (const node of sorted) {
    const column = useLifecycle
      ? (LIFECYCLE_COLUMN[phaseByNode.get(node.id) ?? ""] ?? 2)
      : (KIND_COLUMN[node.kind] ?? 3);
    const row = columnRows.get(column) ?? 0;
    columnRows.set(column, row + 1);
    positions.set(node.id, {
      x: PAD + column * COL_GAP,
      y: PAD + row * ROW_GAP,
    });
  }
  let maxRow = 0;
  for (const row of columnRows.values()) {
    maxRow = Math.max(maxRow, row);
  }
  const width = PAD * 2 + (useLifecycle ? 5 : 6) * COL_GAP + NODE_W;
  const height = PAD * 2 + Math.max(maxRow, 1) * ROW_GAP + NODE_H;
  return { positions, width, height, phaseByNode, useLifecycle };
}

function shortLabel(label: string) {
  return label.length > 16 ? `${label.slice(0, 14)}…` : label;
}

export function DigitalThreadGraph({
  nodes,
  edges,
  deviceLinks = [],
  lifecycleRows = [],
  selectedId: selectedIdProp,
  onSelectNode,
}: Props) {
  const [internalSelected, setInternalSelected] = useState<string | null>(null);
  const selectedId = selectedIdProp ?? internalSelected;

  const { positions, width, height, phaseByNode, useLifecycle } = useMemo(
    () => layoutGraph(nodes, lifecycleRows),
    [nodes, lifecycleRows],
  );

  const neighborIds = useMemo(() => {
    if (!selectedId) {
      return new Set<string>();
    }
    const linked = new Set<string>([selectedId]);
    for (const edge of edges) {
      if (edge.from === selectedId) {
        linked.add(edge.to);
      }
      if (edge.to === selectedId) {
        linked.add(edge.from);
      }
    }
    return linked;
  }, [edges, selectedId]);

  const selectNode = (id: string | null) => {
    if (onSelectNode) {
      onSelectNode(id);
    } else {
      setInternalSelected(id);
    }
  };

  const selectedNode = nodes.find((node) => node.id === selectedId);
  const selectedDevice = deviceLinks.find((link) => link.device_id === selectedId);

  if (nodes.length === 0) {
    return <p className="demo-hint">No graph nodes — load a program with <code>--program</code>.</p>;
  }

  return (
    <div className="digital-thread-graph">
      <div className="digital-thread-graph__canvas" style={{ maxHeight: 420 }}>
        <svg
          width={width}
          height={height}
          viewBox={`0 0 ${width} ${height}`}
          role="img"
          aria-label="Digital thread dependency graph"
        >
          {edges.map((edge) => {
            const from = positions.get(edge.from);
            const to = positions.get(edge.to);
            if (!from || !to) {
              return null;
            }
            const x1 = from.x + NODE_W;
            const y1 = from.y + NODE_H / 2;
            const x2 = to.x;
            const y2 = to.y + NODE_H / 2;
            const midX = (x1 + x2) / 2;
            const active =
              !selectedId || (neighborIds.has(edge.from) && neighborIds.has(edge.to));
            return (
              <path
                key={`${edge.from}-${edge.to}-${edge.relation}`}
                d={`M ${x1} ${y1} C ${midX} ${y1}, ${midX} ${y2}, ${x2} ${y2}`}
                fill="none"
                stroke={active ? "#58a6ff" : "#30363d"}
                strokeWidth={active ? 2 : 1}
                opacity={active ? 0.9 : 0.35}
              />
            );
          })}
          {nodes.map((node) => {
            const pos = positions.get(node.id);
            if (!pos) {
              return null;
            }
            const fill = useLifecycle
              ? (LIFECYCLE_COLORS[phaseByNode.get(node.id) ?? ""] ?? "#64748b")
              : (KIND_COLORS[node.kind] ?? "#64748b");
            const isSelected = node.id === selectedId;
            const dimmed = selectedId !== null && !neighborIds.has(node.id);
            return (
              <g
                key={node.id}
                transform={`translate(${pos.x}, ${pos.y})`}
                style={{ cursor: "pointer", opacity: dimmed ? 0.35 : 1 }}
                onClick={() => selectNode(isSelected ? null : node.id)}
              >
                <rect
                  width={NODE_W}
                  height={NODE_H}
                  rx={6}
                  fill={fill}
                  stroke={isSelected ? "#f0f6fc" : "#0f1419"}
                  strokeWidth={isSelected ? 2 : 1}
                />
                <text
                  x={NODE_W / 2}
                  y={NODE_H / 2 + 4}
                  textAnchor="middle"
                  fontSize={11}
                  fill="#0f1419"
                  fontFamily="system-ui, sans-serif"
                >
                  {shortLabel(node.label)}
                </text>
                <title>{`${useLifecycle ? phaseByNode.get(node.id) ?? node.kind : node.kind}: ${node.label} (${node.id})`}</title>
              </g>
            );
          })}
        </svg>
      </div>
      {selectedNode && (
        <dl className="digital-thread-graph__detail">
          <dt>Selected</dt>
          <dd>
            {useLifecycle ? phaseByNode.get(selectedNode.id) ?? selectedNode.kind : selectedNode.kind} — {selectedNode.label}
          </dd>
          <dt>Id</dt>
          <dd>{selectedNode.id}</dd>
        </dl>
      )}
      {selectedDevice && (
        <dl className="digital-thread-graph__detail">
          <dt>Device</dt>
          <dd>{selectedDevice.device_type}</dd>
          <dt>Robot</dt>
          <dd>{selectedDevice.assigned_robot ?? "—"}</dd>
          <dt>Lifecycle</dt>
          <dd>{selectedDevice.lifecycle_state ?? "—"}</dd>
          <dt>Capabilities</dt>
          <dd>{selectedDevice.related_capabilities.join(", ") || "—"}</dd>
        </dl>
      )}
    </div>
  );
}
