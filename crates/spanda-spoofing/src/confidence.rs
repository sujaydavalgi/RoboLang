//! Confidence filtering and operator confirmation gates for spoofing alerts.

use crate::trace::{SpoofingAlert, SpoofingSeverity};

/// Minimum confidence threshold for trace spoofing alerts (0.0–1.0).
pub fn spoofing_min_confidence() -> f64 {
    // Read SPANDA_SPOOFING_MIN_CONFIDENCE with ML fallback to SPANDA_SPOOFING_ML_MIN_CONFIDENCE.
    //
    // Parameters:
    // None.
    //
    // Returns:
    // Minimum confidence threshold, or 0.0 when unset.
    //
    // Options:
    // `SPANDA_SPOOFING_MIN_CONFIDENCE` — global trace alert threshold.
    // `SPANDA_SPOOFING_ML_MIN_CONFIDENCE` — fallback when global unset.
    //
    // Example:
    // let min = spoofing_min_confidence();

    std::env::var("SPANDA_SPOOFING_MIN_CONFIDENCE")
        .ok()
        .or_else(|| std::env::var("SPANDA_SPOOFING_ML_MIN_CONFIDENCE").ok())
        .and_then(|value| value.parse::<f64>().ok())
        .map(|value| value.clamp(0.0, 1.0))
        .unwrap_or(0.0)
}

/// Drop alerts below the configured confidence threshold.
pub fn apply_spoofing_confidence_filter(alerts: &mut Vec<SpoofingAlert>) -> u32 {
    // Remove low-confidence spoofing alerts before pass/fail and operator gates.
    //
    // Parameters:
    // - `alerts` — mutable alert list from trace analysis
    //
    // Returns:
    // Count of suppressed alerts.
    //
    // Options:
    // Uses `spoofing_min_confidence()`.
    //
    // Example:
    // let suppressed = apply_spoofing_confidence_filter(&mut alerts);

    let min_confidence = spoofing_min_confidence();
    if min_confidence <= 0.0 {
        return 0;
    }
    let before = alerts.len();
    alerts.retain(|alert| alert.confidence >= min_confidence);
    (before.saturating_sub(alerts.len())) as u32
}

/// Return true when high-severity alerts require operator confirmation before destructive action.
pub fn requires_operator_confirmation(alerts: &[SpoofingAlert]) -> bool {
    // Gate destructive tamper responses on operator approval for remaining high-severity alerts.
    //
    // Parameters:
    // - `alerts` — post-filter spoofing alerts
    //
    // Returns:
    // True when Critical/High alerts remain and operator approval is not bypassed.
    //
    // Options:
    // `SPANDA_OPERATOR_APPROVAL=1` bypasses the gate in simulation.
    //
    // Example:
    // if requires_operator_confirmation(&alerts) { defer stop(); }

    if operator_approval_bypassed() {
        return false;
    }
    alerts.iter().any(|alert| {
        matches!(
            alert.severity,
            SpoofingSeverity::High | SpoofingSeverity::Critical
        )
    })
}

fn operator_approval_bypassed() -> bool {
    std::env::var("SPANDA_OPERATOR_APPROVAL")
        .ok()
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn env_lock() -> MutexGuard<'static, ()> {
        ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner())
    }

    #[test]
    fn confidence_filter_suppresses_low_score_alerts() {
        let _guard = env_lock();
        std::env::set_var("SPANDA_SPOOFING_MIN_CONFIDENCE", "0.9");
        let mut alerts = vec![SpoofingAlert {
            sensor: "gps".into(),
            severity: SpoofingSeverity::High,
            confidence: 0.75,
            message: "low".into(),
            evidence: "test".into(),
            sim_time_ms: None,
        }];
        let suppressed = apply_spoofing_confidence_filter(&mut alerts);
        assert_eq!(suppressed, 1);
        assert!(alerts.is_empty());
        std::env::remove_var("SPANDA_SPOOFING_MIN_CONFIDENCE");
    }

    #[test]
    fn operator_confirmation_required_for_high_alerts() {
        let _guard = env_lock();
        std::env::remove_var("SPANDA_OPERATOR_APPROVAL");
        let alerts = vec![SpoofingAlert {
            sensor: "gps".into(),
            severity: SpoofingSeverity::Critical,
            confidence: 0.99,
            message: "spoof".into(),
            evidence: "test".into(),
            sim_time_ms: None,
        }];
        assert!(requires_operator_confirmation(&alerts));
        std::env::set_var("SPANDA_OPERATOR_APPROVAL", "1");
        assert!(!requires_operator_confirmation(&alerts));
        std::env::remove_var("SPANDA_OPERATOR_APPROVAL");
    }
}
