use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Package trust / safety level for deployment gating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SafetyLevel {
    #[default]
    Experimental,
    SimulationOnly,
    HardwareSafe,
    Certified,
}

impl SafetyLevel {
    pub fn all() -> &'static [SafetyLevel] {
        &[
            Self::Experimental,
            Self::SimulationOnly,
            Self::HardwareSafe,
            Self::Certified,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Experimental => "experimental",
            Self::SimulationOnly => "simulation_only",
            Self::HardwareSafe => "hardware_safe",
            Self::Certified => "certified",
        }
    }

    /// Whether this level may control physical actuators on real hardware.
    pub fn can_control_actuators_default(&self) -> bool {
        matches!(self, Self::HardwareSafe | Self::Certified)
    }

    /// Whether packages at this level require manual review before deployment.
    pub fn requires_review_default(&self) -> bool {
        matches!(self, Self::Experimental | Self::SimulationOnly)
    }
}

impl FromStr for SafetyLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "experimental" => Ok(Self::Experimental),
            "simulation_only" => Ok(Self::SimulationOnly),
            "hardware_safe" => Ok(Self::HardwareSafe),
            "certified" => Ok(Self::Certified),
            other => Err(format!("unknown safety level '{other}'")),
        }
    }
}

/// Safety metadata from `[safety]` in spanda.toml.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafetyMetadata {
    #[serde(default)]
    pub level: SafetyLevel,
    #[serde(default = "default_true")]
    pub requires_review: bool,
    #[serde(default)]
    pub can_control_actuators: bool,
}

fn default_true() -> bool {
    true
}

impl Default for SafetyMetadata {
    fn default() -> Self {
        let level = SafetyLevel::Experimental;
        Self {
            level,
            requires_review: level.requires_review_default(),
            can_control_actuators: level.can_control_actuators_default(),
        }
    }
}

impl SafetyMetadata {
    pub fn normalize(&mut self) {
        if self.level == SafetyLevel::default() && !self.can_control_actuators {
            self.requires_review = self.level.requires_review_default();
        }
    }
}
