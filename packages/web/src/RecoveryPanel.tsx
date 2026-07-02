import { useCallback, useState } from "react";

type RecoveryPlan = {
  plan_id: string;
  entity_id: string;
  failure: string;
  risk: string;
};

type RecoveryMetrics = {
  total_recoveries: number;
  success_rate: number;
  recovery_confidence: number;
};

type Props = {
  baseUrl: string;
};

export function RecoveryPanel({ baseUrl }: Props) {
  const [plans, setPlans] = useState<RecoveryPlan[]>([]);
  const [metrics, setMetrics] = useState<RecoveryMetrics | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    setBusy(true);
    setError(null);
    try {
      const [plansRes, metricsRes] = await Promise.all([
        fetch(`${baseUrl}/v1/recovery/plans`),
        fetch(`${baseUrl}/v1/recovery/metrics`),
      ]);
      const plansJson = await plansRes.json();
      const metricsJson = await metricsRes.json();
      setPlans(Array.isArray(plansJson.plans) ? plansJson.plans : []);
      setMetrics(metricsJson.metrics ?? null);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Failed to load recovery data");
    } finally {
      setBusy(false);
    }
  }, [baseUrl]);

  return (
    <section className="recovery-panel">
      <header>
        <h2>Recovery Orchestrator</h2>
        <button type="button" onClick={load} disabled={busy}>
          {busy ? "Loading…" : "Refresh"}
        </button>
      </header>
      {error ? <p className="error">{error}</p> : null}
      {metrics ? (
        <div className="recovery-metrics">
          <span>Recoveries: {metrics.total_recoveries}</span>
          <span>Success: {(metrics.success_rate * 100).toFixed(0)}%</span>
          <span>Confidence: {(metrics.recovery_confidence * 100).toFixed(0)}%</span>
        </div>
      ) : null}
      <h3>Recovery Queue / Plans</h3>
      <ul>
        {plans.map((p) => (
          <li key={p.plan_id}>
            <strong>{p.entity_id}</strong> — {p.failure} (risk: {p.risk})
          </li>
        ))}
        {plans.length === 0 ? <li>No active recovery plans</li> : null}
      </ul>
    </section>
  );
}
