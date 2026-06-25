//! Logical-to-physical device mapping derived from configuration and programs.
//!
use crate::device_identity::{DeviceIdentityRecord, DeviceRegistry};
use crate::device_tree::DeviceTree;
use serde::{Deserialize, Serialize};
use spanda_ast::nodes::{ActuatorDecl, Program, RobotDecl, SensorDecl};
use std::collections::HashMap;

/// Mapping between logical program entities and physical configuration devices.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LogicalPhysicalMap {
    pub robots: HashMap<String, RobotMapping>,
    pub sensors: HashMap<String, SensorMapping>,
    pub actuators: HashMap<String, ActuatorMapping>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RobotMapping {
    pub logical_id: String,
    pub physical_robot_id: String,
    pub hardware_profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorMapping {
    pub logical_name: String,
    pub physical_device_id: String,
    pub robot_id: String,
    pub device_type: String,
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub endpoint_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActuatorMapping {
    pub logical_name: String,
    pub physical_device_id: String,
    pub robot_id: String,
    pub device_type: String,
    pub capabilities: Vec<String>,
    pub has_emergency_stop: bool,
    #[serde(default)]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub endpoint_url: Option<String>,
}

impl LogicalPhysicalMap {
    pub fn from_device_tree_and_registry(tree: &DeviceTree, registry: &DeviceRegistry) -> Self {
        let mut map = Self::from_registry(registry);
        let Some(ref fleet) = tree.fleet else {
            return map;
        };
        for robot in &fleet.robots {
            map.robots.insert(
                robot.id.clone(),
                RobotMapping {
                    logical_id: robot.id.clone(),
                    physical_robot_id: robot.id.clone(),
                    hardware_profile: robot.hardware_profile.clone(),
                },
            );
        }
        map
    }

    pub fn from_device_tree(tree: &DeviceTree) -> Self {
        let registry =
            DeviceRegistry::from_resolved_parts(tree, &toml::Value::Table(Default::default()));
        Self::from_device_tree_and_registry(tree, &registry)
    }

    pub fn from_registry(registry: &DeviceRegistry) -> Self {
        let mut map = Self::default();
        for device in &registry.devices {
            insert_device_mapping(device, &mut map);
        }
        map
    }

    pub fn verify(&self) -> Vec<String> {
        let mut issues = Vec::new();
        issues.extend(self.warnings.clone());
        for (name, actuator) in &self.actuators {
            let is_drive = actuator.device_type.contains("Drive")
                || actuator.device_type.contains("Actuator")
                || actuator.device_type.contains("Motor");
            if is_drive && !actuator.has_emergency_stop {
                issues.push(format!(
                    "actuator '{name}' ({}) missing emergency_stop capability",
                    actuator.physical_device_id
                ));
            }
        }
        issues
    }

    pub fn verify_against_program(
        &self,
        program: &Program,
        registry: &DeviceRegistry,
    ) -> Vec<String> {
        let mut issues = self.verify();
        let Program::Program { robots, .. } = program;
        for robot in robots {
            let RobotDecl::RobotDecl {
                name,
                sensors,
                actuators,
                ..
            } = robot;
            for sensor in sensors {
                let SensorDecl::SensorDecl {
                    name: sensor_name, ..
                } = sensor;
                if !self.sensors.contains_key(sensor_name) {
                    let matched = registry.by_logical_name(sensor_name);
                    if matched.is_empty() {
                        issues.push(format!(
                            "program sensor '{sensor_name}' on robot '{name}' has no configured device mapping"
                        ));
                    }
                }
            }
            for actuator in actuators {
                let ActuatorDecl::ActuatorDecl {
                    name: actuator_name,
                    ..
                } = actuator;
                if !self.actuators.contains_key(actuator_name) {
                    let matched = registry.by_logical_name(actuator_name);
                    if matched.is_empty() {
                        issues.push(format!(
                            "program actuator '{actuator_name}' on robot '{name}' has no configured device mapping"
                        ));
                    }
                }
            }
        }
        issues
    }
}

fn insert_device_mapping(device: &DeviceIdentityRecord, map: &mut LogicalPhysicalMap) {
    let robot_id = device.robot_id.clone().unwrap_or_else(|| "unknown".into());
    let logical = device
        .logical_name
        .clone()
        .unwrap_or_else(|| device.id.clone());
    let dtype = device.device_type.to_ascii_lowercase();
    if dtype.contains("gps")
        || dtype.contains("lidar")
        || dtype.contains("camera")
        || dtype.contains("imu")
        || dtype.contains("sensor")
    {
        map.sensors.insert(
            logical.clone(),
            SensorMapping {
                logical_name: logical,
                physical_device_id: device.id.clone(),
                robot_id,
                device_type: device.device_type.clone(),
                capabilities: device.capabilities.clone(),
                ip_address: device.ip_address.clone(),
                endpoint_url: device.endpoint_url.clone(),
            },
        );
    } else if dtype.contains("drive")
        || dtype.contains("actuator")
        || dtype.contains("arm")
        || dtype.contains("motor")
        || dtype.contains("controller")
    {
        let has_estop = device.capabilities.iter().any(|c| c == "emergency_stop");
        map.actuators.insert(
            logical.clone(),
            ActuatorMapping {
                logical_name: logical,
                physical_device_id: device.id.clone(),
                robot_id,
                device_type: device.device_type.clone(),
                capabilities: device.capabilities.clone(),
                has_emergency_stop: has_estop,
                ip_address: device.ip_address.clone(),
                endpoint_url: device.endpoint_url.clone(),
            },
        );
    }
}
