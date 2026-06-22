//! Message schema registry used during compile-time type checking.
//!
use spanda_ast::comm_decl::{MessageDecl, MessageSchema};
use spanda_ast::foundations::StructDecl;
use spanda_ast::nodes::SpandaType;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub struct MessageRegistry {
    schemas: HashMap<String, MessageSchema>,
    builtin: HashSet<String>,
}

impl MessageRegistry {
    pub fn new() -> Self {
        // Seed built-in message type names used by comm declarations.
        let mut reg = Self::default();
        for name in ["Velocity", "Pose", "Scan", "String"] {
            reg.builtin.insert(name.into());
        }
        reg
    }

    pub fn register(&mut self, decl: &MessageDecl) {
        let MessageDecl::MessageDecl {
            name,
            fields,
            version,
            ..
        } = decl;
        self.schemas.insert(
            name.clone(),
            MessageSchema {
                name: name.clone(),
                fields: fields
                    .iter()
                    .map(|f| (f.name.clone(), f.type_name.clone()))
                    .collect(),
                version: *version,
            },
        );
    }

    pub fn from_program(messages: &[MessageDecl], structs: &[StructDecl]) -> Self {
        let mut reg = Self::new();
        for msg in messages {
            reg.register(msg);
        }
        for s in structs {
            let StructDecl::StructDecl { name, fields, .. } = s;
            reg.schemas.insert(
                name.clone(),
                MessageSchema {
                    name: name.clone(),
                    fields: fields
                        .iter()
                        .map(|f| (f.name.clone(), f.type_name.clone()))
                        .collect(),
                    version: None,
                },
            );
        }
        reg
    }

    pub fn is_known(&self, name: &str) -> bool {
        self.builtin.contains(name) || self.schemas.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&MessageSchema> {
        self.schemas.get(name)
    }

    pub fn resolve_type(&self, name: &str) -> Option<SpandaType> {
        match name {
            "Velocity" => Some(SpandaType::Velocity),
            "Pose" => Some(SpandaType::Pose),
            "Scan" => Some(SpandaType::Scan),
            "String" => Some(SpandaType::String),
            "Command" | "Conversation" | "Feedback" | "Approval" | "Intent" => {
                Some(SpandaType::Named { name: name.into() })
            }
            "SafeMessage" | "VerifiedMessage" | "TrustedSource" | "ActionProposal"
            | "SafeAction" | "CommandMessage" | "EncryptedMessage" | "SignedMessage"
            | "Certificate" | "PublicKey" | "PrivateKey" | "SessionKey" => {
                Some(SpandaType::Named { name: name.into() })
            }
            "BatteryRequest" | "BatteryStatus" | "NavigationFeedback" | "NavigationResult"
            | "LidarReading" | "LidarScan" | "Timestamp" | "PathPlan" => {
                Some(SpandaType::Named { name: name.into() })
            }
            other if self.schemas.contains_key(other) => {
                Some(SpandaType::Named { name: other.into() })
            }
            other
                if other.starts_with("Topic<")
                    || other.starts_with("Service<")
                    || other.starts_with("Action<") =>
            {
                Some(SpandaType::Named { name: other.into() })
            }
            _ => None,
        }
    }
}

/// Agent capability actions that map to comm-bus operations.
pub const COMM_CAPABILITIES: &[&str] = &["subscribe", "publish", "call", "execute", "discover"];

pub fn is_comm_capability(action: &str) -> bool {
    COMM_CAPABILITIES.contains(&action)
}
