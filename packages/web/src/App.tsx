import { useCallback, useEffect, useState } from "react";
import { DEFAULT_SOURCE, EXAMPLES } from "./examples";
import { checkSource, runSource, type CheckResponse, type RunResponse } from "./spanda-wasm";
import { ControlCenterPanel } from "./ControlCenterPanel";
import { OperationsPanel } from "./OperationsPanel";
import { TelemetryPanel } from "./TelemetryPanel";

type Backend = "wasm" | "unavailable";
type View = "playground" | "operations" | "control-center";

export default function App() {
  const [source, setSource] = useState(DEFAULT_SOURCE);
  const [view, setView] = useState<View>("playground");
  const [backend, setBackend] = useState<Backend>("unavailable");
  const [diagnostics, setDiagnostics] = useState<CheckResponse["diagnostics"]>([]);
  const [runResult, setRunResult] = useState<RunResponse["result"] | null>(null);
  const [telemetryRefresh, setTelemetryRefresh] = useState(0);
  const [apiBase, setApiBase] = useState(
    () => import.meta.env.VITE_CONTROL_CENTER_URL ?? "http://127.0.0.1:8080"
  );
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    import("./spanda-wasm").then((m) => {
      setBackend(m.isWasmLoaded() ? "wasm" : "unavailable");
    });
  }, []);

  const handleCheck = useCallback(async () => {
    setBusy(true);
    setError(null);
    setRunResult(null);
    try {
      const resp = await checkSource(source);
      setDiagnostics(resp.diagnostics);
      if (!resp.ok) setError("Type check failed");
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }, [source]);

  const handleRun = useCallback(async () => {
    setBusy(true);
    setError(null);
    try {
      const resp = await runSource(source, 10);
      setDiagnostics(resp.diagnostics ?? []);
      if (resp.ok && resp.result) {
        setRunResult(resp.result);
        setTelemetryRefresh((value) => value + 1);
      } else {
        setError(resp.diagnostics?.[0]?.message ?? "Run failed");
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }, [source]);

  return (
    <div className="app">
      <header>
        <div>
          <h1>Spanda Playground</h1>
          <p className="subtitle">Flagship killer demo — Check + Run sim via Rust WASM (no local CLI)</p>
        </div>
        <span className={`badge ${backend}`}>
          {backend === "wasm" ? "Rust WASM" : "WASM not built"}
        </span>
      </header>

      <div className="toolbar">
        <select
          value={view}
          onChange={(e) => setView(e.target.value as View)}
          aria-label="View"
        >
          <option value="playground">Playground</option>
          <option value="operations">Operations</option>
          <option value="control-center">Control Center</option>
        </select>
        {view === "control-center" && (
          <input
            type="url"
            value={apiBase}
            onChange={(e) => setApiBase(e.target.value)}
            aria-label="Control Center API URL"
            style={{ minWidth: "16rem" }}
          />
        )}
        <select
          onChange={(e) => {
            const ex = EXAMPLES.find((x) => x.name === e.target.value);
            if (ex) setSource(ex.source);
          }}
          defaultValue="Killer demo (flagship)"
        >
          {EXAMPLES.map((ex) => (
            <option key={ex.name} value={ex.name}>
              {ex.name}
            </option>
          ))}
        </select>
        <span className="demo-hint">
          {view === "playground"
            ? "Check → Run sim (verify needs native CLI)"
            : view === "control-center"
              ? "Dashboard · Fleet · Readiness via spanda control-center serve"
              : "Readiness scoring + live agent"}
        </span>
        {view === "playground" && (
          <>
            <button type="button" onClick={handleCheck} disabled={busy || backend === "unavailable"}>
              Check
            </button>
            <button type="button" className="primary" onClick={handleRun} disabled={busy || backend === "unavailable"}>
              Run sim
            </button>
          </>
        )}
      </div>

      {backend === "unavailable" && (
        <div className="banner">
          Build WASM: <code>./scripts/build-wasm.sh</code> then refresh.
        </div>
      )}

      <div className="layout">
        <section className="editor-pane">
          <textarea
            value={source}
            onChange={(e) => setSource(e.target.value)}
            spellCheck={false}
            aria-label="Spanda source"
          />
        </section>

        <section className="output-pane">
          {view === "control-center" ? (
            <ControlCenterPanel apiBase={apiBase} />
          ) : view === "operations" ? (
            <>
              <OperationsPanel source={source} />
              <TelemetryPanel refreshKey={telemetryRefresh} />
            </>
          ) : (
            <>
              {error && <div className="error">{error}</div>}

              {diagnostics.length > 0 && (
                <div className="panel">
                  <h2>Diagnostics</h2>
                  <ul>
                    {diagnostics.map((d, i) => (
                      <li key={i}>
                        [{d.line}:{d.column}] {d.message}
                      </li>
                    ))}
                  </ul>
                </div>
              )}

              {runResult && (
                <>
                  <div className="panel">
                    <h2>Robot state</h2>
                    <dl>
                      <dt>Pose</dt>
                      <dd>
                        x={runResult.state.pose.x.toFixed(3)} m, y={runResult.state.pose.y.toFixed(3)} m, θ=
                        {runResult.state.pose.theta.toFixed(3)} rad
                      </dd>
                      <dt>Velocity</dt>
                      <dd>
                        {runResult.state.velocity.linear.toFixed(3)} m/s, {runResult.state.velocity.angular.toFixed(3)}{" "}
                        rad/s
                      </dd>
                      <dt>E-stop</dt>
                      <dd>{runResult.state.emergency_stop ? "ACTIVE" : "off"}</dd>
                    </dl>
                  </div>
                  <div className="panel">
                    <h2>Simulation log</h2>
                    <pre>{runResult.events.join("\n") || "(no events)"}</pre>
                  </div>
                  <TelemetryPanel refreshKey={telemetryRefresh} />
                </>
              )}
            </>
          )}
        </section>
      </div>
    </div>
  );
}
