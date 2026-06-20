use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebugCommand {
    Continue,
    Step,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugPause {
    pub line: u32,
    pub reason: String,
}

#[derive(Debug, Clone, Default)]
pub struct DebugOptions {
    pub breakpoints: HashSet<u32>,
    pub step: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSession {
    pub pauses: Vec<DebugPause>,
}

impl DebugSession {
    pub fn paused(&self) -> bool {
        !self.pauses.is_empty()
    }
}

#[derive(Clone)]
pub struct DebugController {
    breakpoints: HashSet<u32>,
    step: RefCell<bool>,
    pauses: Rc<RefCell<Vec<DebugPause>>>,
}

impl DebugController {
    pub fn new(options: DebugOptions) -> Self {
        Self {
            breakpoints: options.breakpoints,
            step: RefCell::new(options.step),
            pauses: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn pauses(&self) -> Rc<RefCell<Vec<DebugPause>>> {
        self.pauses.clone()
    }

    pub fn should_pause(&self, line: u32) -> bool {
        if *self.step.borrow() {
            *self.step.borrow_mut() = false;
            return true;
        }
        self.breakpoints.contains(&line)
    }

    pub fn record_pause(&self, line: u32, reason: &str) {
        self.pauses.borrow_mut().push(DebugPause {
            line,
            reason: reason.to_string(),
        });
    }

    pub fn command(&self, cmd: DebugCommand) {
        if matches!(cmd, DebugCommand::Step) {
            *self.step.borrow_mut() = true;
        }
    }
}

pub fn stmt_line(stmt: &crate::ast::Stmt) -> u32 {
    use crate::ast::Stmt;
    match stmt {
        Stmt::VarDecl { span, .. }
        | Stmt::IfStmt { span, .. }
        | Stmt::LoopStmt { span, .. }
        | Stmt::ExprStmt { span, .. }
        | Stmt::ReturnStmt { span, .. }
        | Stmt::PublishStmt { span, .. }
        | Stmt::ServiceCallStmt { span, .. }
        | Stmt::ActionSendStmt { span, .. }
        | Stmt::EmergencyStopStmt { span, .. }
        | Stmt::ResetEmergencyStopStmt { span, .. }
        | Stmt::EmitStmt { span, .. }
        | Stmt::EnterStmt { span, .. }
        | Stmt::RememberStmt { span, .. }
        | Stmt::SubscribeStmt { span, .. }
        | Stmt::ExecuteStmt { span, .. }
        | Stmt::DiscoverStmt { span, .. }
        | Stmt::ReceiveStmt { span, .. }
        | Stmt::SpawnStmt { span, .. }
        | Stmt::SelectStmt { span, .. } => span.start.line,
    }
}
