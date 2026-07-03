//! Local decision tree extraction and evaluation.

use crate::types::DecisionLayer;
use serde::{Deserialize, Serialize};
use spanda_ast::assurance_decl::{DecisionTreeBranch, DecisionTreeDecl};
use spanda_ast::nodes::Program;
use spanda_audit::{sign, verify_signature};

/// Evaluated branch result from a decision tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionTreeResult {
    pub tree_name: String,
    pub layer: DecisionLayer,
    pub condition_matched: String,
    pub actions: Vec<String>,
    pub version: String,
    pub tree_hash: String,
}

/// Serializable decision tree specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionTreeSpec {
    pub name: String,
    pub scope: String,
    pub layer: DecisionLayer,
    pub version: String,
    pub branches: Vec<DecisionTreeBranch>,
    pub signature: Option<String>,
}

/// Extract decision trees from a program AST.
pub fn extract_decision_trees(program: &Program) -> Vec<DecisionTreeSpec> {
    let Program::Program { decision_trees, .. } = program;
    decision_trees
        .iter()
        .map(|decl| {
            let DecisionTreeDecl::DecisionTreeDecl {
                name,
                scope,
                layer,
                branches,
                version,
                signature,
                ..
            } = decl;
            DecisionTreeSpec {
                name: name.clone(),
                scope: scope.clone(),
                layer: parse_tree_layer(layer),
                version: version.clone().unwrap_or_else(|| "1".into()),
                branches: branches.clone(),
                signature: signature.clone(),
            }
        })
        .collect()
}

/// Resolve decision trees merging program definitions with persisted signed cache.
pub fn resolve_decision_trees(program: &Program) -> Vec<DecisionTreeSpec> {
    let trees = extract_decision_trees(program);
    let cache = crate::policy_cache::load_persisted_policy_cache(None);
    crate::policy_cache::merge_decision_trees_with_cache(trees, &cache)
}

fn parse_tree_layer(layer: &str) -> DecisionLayer {
    match layer {
        "reflex" | "local_reflex" => DecisionLayer::Reflex,
        "fleet" | "group" | "swarm" => DecisionLayer::GroupFleet,
        "central" | "cloud" | "control_center" => DecisionLayer::ControlCenter,
        _ => DecisionLayer::LocalEntity,
    }
}

/// Map decision layer string to conflict precedence key.
pub fn layer_str_precedence_key(layer: &str) -> &'static str {
    let lower = layer.to_lowercase();
    if lower.contains("kill") {
        "safety_kill_switch"
    } else if lower.contains("reflex") {
        "local_immediate_safety"
    } else if lower.contains("fleet") || lower.contains("group") {
        "fleet_coordination"
    } else if lower.contains("control") {
        "control_center_policy"
    } else {
        "local_optimization"
    }
}

/// Map decision layer to conflict precedence key.
pub fn layer_precedence_key(layer: DecisionLayer) -> &'static str {
    match layer {
        DecisionLayer::Reflex => "local_immediate_safety",
        DecisionLayer::LocalEntity => "local_optimization",
        DecisionLayer::GroupFleet => "fleet_coordination",
        DecisionLayer::ControlCenter => "control_center_policy",
    }
}

/// Simple hash fingerprint for tamper detection (fast lookup).
pub fn tree_hash(spec: &DecisionTreeSpec) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    spec.name.hash(&mut hasher);
    spec.version.hash(&mut hasher);
    for b in &spec.branches {
        b.condition.hash(&mut hasher);
        for a in &b.actions {
            a.hash(&mut hasher);
        }
    }
    format!("{:016x}", hasher.finish())
}

/// Canonical signing payload for a decision tree (stable field order).
pub fn decision_tree_signing_payload(spec: &DecisionTreeSpec) -> String {
    let branches: Vec<_> = spec
        .branches
        .iter()
        .map(|b| {
            serde_json::json!({
                "condition": b.condition,
                "actions": b.actions,
            })
        })
        .collect();
    serde_json::json!({
        "name": spec.name,
        "scope": spec.scope,
        "version": spec.version,
        "branches": branches,
    })
    .to_string()
}

/// Sign a decision tree with a trust key.
pub fn sign_decision_tree(spec: &DecisionTreeSpec, signing_key: &str) -> String {
    sign(&decision_tree_signing_payload(spec), signing_key)
}

/// Verify a decision tree Ed25519 signature.
pub fn verify_decision_tree_signature(spec: &DecisionTreeSpec, trust_key: &str) -> bool {
    let Some(signature) = spec.signature.as_ref().filter(|s| !s.is_empty()) else {
        return false;
    };
    verify_signature(&decision_tree_signing_payload(spec), signature, trust_key)
}

fn require_signed_decision_trees() -> bool {
    std::env::var("SPANDA_DECISION_REQUIRE_SIGNED_TREES")
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

fn decision_tree_trust_key() -> Option<String> {
    std::env::var("SPANDA_DECISION_POLICY_TRUST_KEY")
        .ok()
        .filter(|key| !key.is_empty())
}

/// Validate decision tree trust before evaluation.
pub fn validate_decision_tree_trust(spec: &DecisionTreeSpec) -> Result<(), String> {
    let require = require_signed_decision_trees();
    let has_signature = spec
        .signature
        .as_ref()
        .map(|s| !s.is_empty())
        .unwrap_or(false);

    if !require && !has_signature {
        return Ok(());
    }

    if !has_signature {
        return Err(format!(
            "decision tree '{}' requires a signed cache entry",
            spec.name
        ));
    }

    let trust_key = decision_tree_trust_key().ok_or_else(|| {
        "SPANDA_DECISION_POLICY_TRUST_KEY not configured for decision tree verification"
            .to_string()
    })?;

    if !verify_decision_tree_signature(spec, &trust_key) {
        return Err(format!(
            "decision tree '{}' signature verification failed",
            spec.name
        ));
    }

    Ok(())
}

/// Evaluate a decision tree against a signal map (condition key → bool).
pub fn evaluate_tree(
    spec: &DecisionTreeSpec,
    signals: &std::collections::HashMap<String, bool>,
) -> Option<DecisionTreeResult> {
    if validate_decision_tree_trust(spec).is_err() {
        return None;
    }

    for branch in &spec.branches {
        if signals.get(&branch.condition).copied().unwrap_or(false) {
            for nested in &branch.nested {
                if nested.condition == "else"
                    || signals.get(&nested.condition).copied().unwrap_or(false)
                {
                    return Some(DecisionTreeResult {
                        tree_name: spec.name.clone(),
                        layer: spec.layer,
                        condition_matched: nested.condition.clone(),
                        actions: nested.actions.clone(),
                        version: spec.version.clone(),
                        tree_hash: tree_hash(spec),
                    });
                }
            }
            if !branch.actions.is_empty() {
                return Some(DecisionTreeResult {
                    tree_name: spec.name.clone(),
                    layer: spec.layer,
                    condition_matched: branch.condition.clone(),
                    actions: branch.actions.clone(),
                    version: spec.version.clone(),
                    tree_hash: tree_hash(spec),
                });
            }
        }
    }
    None
}
