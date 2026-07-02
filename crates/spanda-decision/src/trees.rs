//! Local decision tree extraction and evaluation.

use crate::types::DecisionLayer;
use serde::{Deserialize, Serialize};
use spanda_ast::assurance_decl::{DecisionTreeBranch, DecisionTreeDecl};
use spanda_ast::nodes::Program;

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
                ..
            } = decl;
            DecisionTreeSpec {
                name: name.clone(),
                scope: scope.clone(),
                layer: parse_tree_layer(layer),
                version: version.clone().unwrap_or_else(|| "1".into()),
                branches: branches.clone(),
            }
        })
        .collect()
}

/// Serializable decision tree specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionTreeSpec {
    pub name: String,
    pub scope: String,
    pub layer: DecisionLayer,
    pub version: String,
    pub branches: Vec<DecisionTreeBranch>,
}

fn parse_tree_layer(layer: &str) -> DecisionLayer {
    match layer {
        "reflex" | "local_reflex" => DecisionLayer::Reflex,
        "fleet" | "group" | "swarm" => DecisionLayer::GroupFleet,
        "central" | "cloud" | "control_center" => DecisionLayer::ControlCenter,
        _ => DecisionLayer::LocalEntity,
    }
}

/// Simple hash fingerprint for tamper detection (not cryptographic).
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

/// Evaluate a decision tree against a signal map (condition key → bool).
pub fn evaluate_tree(
    spec: &DecisionTreeSpec,
    signals: &std::collections::HashMap<String, bool>,
) -> Option<DecisionTreeResult> {
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
