//! Decision runtime bridge for CLI and API run options.

use spanda_decision::DecisionBackedRuntime;
use spanda_runtime::decision_runtime::SharedDecisionRuntime;

/// Default distributed decision runtime backed by `spanda-decision`.
pub fn default_decision_runtime() -> SharedDecisionRuntime {
    std::sync::Arc::new(DecisionBackedRuntime)
}
