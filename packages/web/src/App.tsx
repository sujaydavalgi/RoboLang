import { useCallback, useEffect, useState } from "react";
import { DEFAULT_SOURCE, EXAMPLES } from "./examples";
import { checkSource, runSource, type CheckResponse, type RunResponse } from "./synapse-wasm";

type Backend = "wasm" | "unavailable";

export default function App() {
  const [source, setSource] = useState(DEFAULT_SOURCE);
  const [backend, setBackend] = useState<Backend>("unavailable");
  const [diagnostics, setDiagnostics] = useState<CheckResponse["diagnostics"]>([]);
  const [runResult, setRunResult] = useState<RunResponse["result"] | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    import("./synapse-wasm").then((m) => {
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
          <h1>Synapse Playground</h1>
          <p className="subtitle">AI-native robotics language — Rust core via WASM</p>
        </div>
        <span className={`badge ${backend}`}>
          {backend === "wasm" ? "Rust WASM" : "WASM not built"}
        </span>
      </header>

      <div className="toolbar">
        <select
          onChange={(e) => {
            const ex = EXAMPLES.find((x) => x.name === e.target.value);
            if (ex) setSource(ex.source);
          }}
          defaultValue=""
        >
          <option value="" disabled>
            Load example…
          </option>
          {EXAMPLES.map((ex) => (
            <option key={ex.name} value={ex.name}>
              {ex.name}
            </option>
          ))}
        </select>
        <button type="button" onClick={handleCheck} disabled={busy || backend === "unavailable"}>
          Check
        </button>
        <button type="button" className="primary" onClick={handleRun} disabled={busy || backend === "unavailable"}>
          Run sim
        </button>
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
            aria-label="Synapse source"
          />
        </section>

        <section className="output-pane">
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
            </>
          )}
        </section>
      </div>
    </div>
  );
}
