//! Kill switch registration and remote-signed activation enforcement.

use super::{Interpreter, RobotBackend};
use spanda_ast::foundations::KillSwitchDecl;
use spanda_ast::nodes::Program;
use spanda_error::SpandaError;
use spanda_security::signed::SignedMessage;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn cache_kill_switches(&mut self, program: &Program) {
        // Cache kill switch declarations for runtime activation checks.
        //
        // Parameters:
        // - `self` — method receiver
        // - `program` — parsed program
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.cache_kill_switches(program);

        self.kill_switch_defs.clear();
        let Program::Program {
            kill_switches,
            robots,
            ..
        } = program;
        for ks in kill_switches {
            let KillSwitchDecl::KillSwitchDecl { name, .. } = ks;
            self.kill_switch_defs.insert(name.clone(), ks.clone());
        }
        for robot in robots {
            let spanda_ast::nodes::RobotDecl::RobotDecl {
                kill_switches: robot_kill_switches,
                ..
            } = robot;
            for ks in robot_kill_switches {
                let KillSwitchDecl::KillSwitchDecl { name, .. } = ks;
                self.kill_switch_defs.insert(name.clone(), ks.clone());
            }
        }
    }

    pub(super) fn activate_kill_switch(&mut self, name: &str) -> Result<(), SpandaError> {
        // Activate a kill switch after optional remote signature verification.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — kill switch name
        //
        // Returns:
        // Ok when activation succeeds, or an error when verification fails.
        //
        // Options:
        // - `RunOptions::kill_switch_signature` supplies JSON for remote_signed switches.
        //
        // Example:
        // interpreter.activate_kill_switch("EmergencyStop")?;

        let Some(decl) = self.kill_switch_defs.get(name).cloned() else {
            return Err(SpandaError::Runtime {
                message: format!("Unknown kill switch '{name}'"),
                line: 1,
            });
        };
        let KillSwitchDecl::KillSwitchDecl {
            remote_signed,
            body,
            ..
        } = decl;

        // Require a verified signature when the switch is marked remote_signed.
        if remote_signed {
            let signature_json = self
                .options
                .kill_switch_signature
                .as_deref()
                .ok_or_else(|| SpandaError::Runtime {
                    message: format!(
                        "Kill switch '{name}' requires remote_signed activation but no signature was provided"
                    ),
                    line: 1,
                })?;
            let signed: SignedMessage = serde_json::from_str(signature_json).map_err(|e| {
                SpandaError::Runtime {
                    message: format!("Invalid kill switch signature JSON: {e}"),
                    line: 1,
                }
            })?;
            let identity = self.security.identity.as_ref().ok_or_else(|| SpandaError::Runtime {
                message: "Kill switch remote_signed requires robot identity".into(),
                line: 1,
            })?;
            if !signed.verify(identity).unwrap_or(false) {
                return Err(SpandaError::Runtime {
                    message: format!("Kill switch '{name}' signature verification failed"),
                    line: 1,
                });
            }
            self.log(format!("kill_switch: verified remote signature for {name}"));
        }

        self.backend.set_emergency_stop(true);
        self.log(format!("kill_switch: activated {name}"));
        self.record_debug_event(
            1,
            "kill_switch_activated",
            &[("kill_switch", name.to_string())],
        );
        for stmt in &body {
            let _ = self.execute_stmt(stmt);
        }
        let _ = self.dispatch_kill_switch_handlers(name);
        Ok(())
    }

    pub(super) fn dispatch_kill_switch_handlers(
        &mut self,
        name: &str,
    ) -> Result<(), SpandaError> {
        // Dispatch registered `on kill_switch` trigger handlers.
        //
        // Parameters:
        // - `self` — method receiver
        // - `name` — kill switch name
        //
        // Returns:
        // Ok when handlers complete.
        //
        // Options:
        // None.
        //
        // Example:
        // interpreter.dispatch_kill_switch_handlers("EmergencyStop")?;

        let handlers: Vec<_> = self
            .trigger_registry
            .handlers_for_kill_switch(name)
            .into_iter()
            .cloned()
            .collect();
        for handler in handlers {
            self.execute_block(&handler.body)?;
        }
        Ok(())
    }
}
