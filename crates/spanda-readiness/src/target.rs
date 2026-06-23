//! Deploy target inference for readiness evaluation.

use spanda_ast::foundations::DeployDecl;
use spanda_ast::nodes::Program;

/// Return the first `deploy … to <target>` hardware profile name when present.
pub fn default_deploy_target(program: &Program) -> Option<String> {
    let Program::Program { deployments, .. } = program;
    deployments.first().and_then(|deploy| {
        let DeployDecl::DeployDecl { targets, .. } = deploy;
        targets.first().cloned()
    })
}

/// Build readiness options from CLI-style flags and program deploy metadata.
pub fn readiness_options_from_flags(
    program: &Program,
    target_flag: Option<String>,
    include_runtime: bool,
    inject_health_faults: bool,
    simulate: bool,
    strict: bool,
) -> crate::types::ReadinessOptions {
    let target = target_flag.or_else(|| default_deploy_target(program));
    crate::types::ReadinessOptions {
        target,
        policy: None,
        simulate,
        strict,
        include_runtime,
        inject_health_faults,
    }
}
