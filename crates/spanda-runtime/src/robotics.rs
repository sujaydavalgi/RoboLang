//! Mission lifecycle and fleet grouping runtime state.
//!
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mission lifecycle states tracked at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum MissionState {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
}

impl MissionState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Running => "Running",
            Self::Paused => "Paused",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
        }
    }
}

/// Runtime mission controller for named step sequences.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MissionRuntime {
    pub name: Option<String>,
    pub steps: Vec<String>,
    pub state: MissionState,
    pub step_index: usize,
    pub duration_hours: Option<f64>,
}

impl MissionRuntime {
    pub fn new(name: Option<String>, steps: Vec<String>, duration_hours: Option<f64>) -> Self {
        Self {
            name,
            steps,
            state: MissionState::Pending,
            step_index: 0,
            duration_hours,
        }
    }

    pub fn start(&mut self) {
        if self.state == MissionState::Pending {
            self.state = MissionState::Running;
        }
    }

    pub fn pause(&mut self) {
        if self.state == MissionState::Running {
            self.state = MissionState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == MissionState::Paused {
            self.state = MissionState::Running;
        }
    }

    pub fn advance(&mut self) -> Option<String> {
        if self.state != MissionState::Running {
            return None;
        }
        if self.step_index >= self.steps.len() {
            self.state = MissionState::Completed;
            return None;
        }
        let step = self.steps[self.step_index].clone();
        self.step_index += 1;
        if self.step_index >= self.steps.len() {
            self.state = MissionState::Completed;
        }
        Some(step)
    }

    pub fn complete(&mut self) {
        self.state = MissionState::Completed;
        self.step_index = self.steps.len();
    }

    pub fn fail(&mut self) {
        self.state = MissionState::Failed;
    }

    pub fn current_step(&self) -> Option<&str> {
        if self.state != MissionState::Running {
            return None;
        }
        self.steps.get(self.step_index).map(String::as_str)
    }
}

/// Registry of fleet groups declared at program scope.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FleetRegistry {
    fleets: HashMap<String, Vec<String>>,
}

impl FleetRegistry {
    pub fn register(&mut self, name: &str, members: Vec<String>) {
        self.fleets.insert(name.to_string(), members);
    }

    pub fn members(&self, name: &str) -> Option<&[String]> {
        self.fleets.get(name).map(Vec::as_slice)
    }

    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.fleets.keys()
    }
}

/// Program-level safety zone speed policies keyed by zone name.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProgramSafetyZoneRegistry {
    zones: HashMap<String, f64>,
}

impl ProgramSafetyZoneRegistry {
    pub fn register(&mut self, name: &str, max_speed_mps: f64) {
        self.zones.insert(name.to_string(), max_speed_mps);
    }

    pub fn max_speed_for(&self, zone_name: &str) -> Option<f64> {
        self.zones.get(zone_name).copied()
    }

    pub fn speed_caps(&self) -> &HashMap<String, f64> {
        &self.zones
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mission_advances_through_steps() {
        let mut mission = MissionRuntime::new(
            Some("Delivery".into()),
            vec!["navigate".into(), "deliver".into()],
            Some(0.5),
        );
        mission.start();
        assert_eq!(mission.advance(), Some("navigate".into()));
        assert_eq!(mission.advance(), Some("deliver".into()));
        assert_eq!(mission.state, MissionState::Completed);
    }

    #[test]
    fn fleet_registry_resolves_members() {
        let mut reg = FleetRegistry::default();
        reg.register("alpha", vec!["r1".into(), "r2".into()]);
        assert_eq!(reg.members("alpha"), Some(["r1".into(), "r2".into()].as_slice()));
    }
}
