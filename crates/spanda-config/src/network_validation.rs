//! Network and device identity validation rules.
//!
use crate::device_identity::{
    check_ip_reachable, logical_name_index, provider_protocol_mismatch, redundant_groups,
    DeviceIdentityRecord, DeviceRegistry,
};
use crate::validation::{ConfigValidationReport, ValidationSeverity};
use spanda_package::adapter::framework_packages;
use std::collections::HashMap;

/// Validate device identity registry for duplicates, endpoints, and redundancy rules.
pub fn validate_device_registry(
    registry: &DeviceRegistry,
    providers: &[String],
) -> ConfigValidationReport {
    let mut report = ConfigValidationReport {
        passed: true,
        findings: Vec::new(),
    };
    let known_providers: std::collections::HashSet<&str> = framework_packages()
        .iter()
        .map(|p| p.name)
        .chain(providers.iter().map(String::as_str))
        .collect();

    let mut ips: HashMap<String, String> = HashMap::new();
    let mut macs: HashMap<String, String> = HashMap::new();
    let mut serials: HashMap<String, String> = HashMap::new();

    for device in &registry.devices {
        let path = format!("devices.{}", device.id);
        if let Some(ref provider) = device.provider {
            if !known_providers.contains(provider.as_str()) {
                report.push(
                    ValidationSeverity::Error,
                    "provider.unknown",
                    format!(
                        "device '{}' references unknown provider '{provider}'",
                        device.id
                    ),
                    Some(format!("{path}.provider")),
                );
            }
            if let Some(ref protocol) = device.protocol {
                if provider_protocol_mismatch(provider, protocol) {
                    report.push(
                        ValidationSeverity::Warning,
                        "device.protocol_provider_mismatch",
                        format!(
                            "device '{}' protocol '{protocol}' may not match provider '{provider}'",
                            device.id
                        ),
                        Some(format!("{path}.protocol")),
                    );
                }
            }
        }

        if let Some(ref ip) = device.ip_address {
            if let Some(other) = ips.insert(ip.clone(), device.id.clone()) {
                report.push(
                    ValidationSeverity::Error,
                    "device.duplicate_ip",
                    format!("duplicate IP '{ip}' on '{other}' and '{}'", device.id),
                    Some(format!("{path}.ip_address")),
                );
            }
            if std::env::var("SPANDA_CONFIG_PROBE_NETWORK").ok().as_deref() == Some("1") {
                let probe_port = device.port.unwrap_or(80);
                if !check_ip_reachable(ip, probe_port, 300) {
                    report.push(
                        ValidationSeverity::Warning,
                        "device.ip_unreachable",
                        format!(
                            "configured IP '{ip}' for device '{}' was not reachable on port {probe_port}",
                            device.id
                        ),
                        Some(format!("{path}.ip_address")),
                    );
                }
            }
        }

        if let Some(ref mac) = device.normalized_mac() {
            if let Some(other) = macs.insert(mac.clone(), device.id.clone()) {
                report.push(
                    ValidationSeverity::Error,
                    "device.duplicate_mac",
                    format!("duplicate MAC '{mac}' on '{other}' and '{}'", device.id),
                    Some(format!("{path}.mac_address")),
                );
            }
        }

        if let Some(ref serial) = device.serial {
            if let Some(other) = serials.insert(serial.clone(), device.id.clone()) {
                report.push(
                    ValidationSeverity::Error,
                    "device.duplicate_serial",
                    format!(
                        "duplicate serial '{serial}' on '{other}' and '{}'",
                        device.id
                    ),
                    Some(format!("{path}.serial")),
                );
            }
        }

        if device.is_networked() && device.endpoint_url.is_none() && device.ip_address.is_none() {
            report.push(
                ValidationSeverity::Warning,
                "device.endpoint_missing",
                format!(
                    "network device '{}' missing endpoint_url or ip_address",
                    device.id
                ),
                Some(path.clone()),
            );
        }

        if device.endpoint_is_insecure() {
            report.push(
                ValidationSeverity::Warning,
                "device.insecure_endpoint",
                format!(
                    "device '{}' endpoint uses an insecure scheme; prefer TLS or signed transport",
                    device.id
                ),
                Some(format!("{path}.endpoint_url")),
            );
        }

        if device.is_remote_actuator()
            && device.endpoint_is_insecure()
            && device.certificate_fingerprint.is_none()
        {
            report.push(
                ValidationSeverity::Error,
                "device.remote_actuator_insecure",
                format!(
                    "remote actuator '{}' requires encrypted/signed endpoint or certificate_fingerprint",
                    device.id
                ),
                Some(format!("{path}.endpoint_url")),
            );
        }

        if device.is_networked()
            && device.security_identity.is_none()
            && device.certificate_fingerprint.is_none()
        {
            report.push(
                ValidationSeverity::Warning,
                "security.identity_missing",
                format!("networked device '{}' has no security identity", device.id),
                Some(path.clone()),
            );
        }
    }

    for (logical, ids) in logical_name_index(registry) {
        if ids.len() > 1 {
            let redundant: Vec<&DeviceIdentityRecord> = registry
                .devices
                .iter()
                .filter(|d| ids.contains(&d.id))
                .collect();
            let all_redundant = redundant
                .iter()
                .all(|d| d.redundant_group.is_some() && d.failover_priority.is_some());
            if !all_redundant {
                report.push(
                    ValidationSeverity::Error,
                    "mapping.logical_ambiguous",
                    format!(
                        "logical_name '{logical}' maps to multiple devices ({}) without redundant_group + failover_priority",
                        ids.join(", ")
                    ),
                    Some(format!("devices.logical_name.{logical}")),
                );
            }
        }
    }

    for (group, members) in redundant_groups(registry) {
        if members.len() < 2 {
            report.push(
                ValidationSeverity::Warning,
                "device.redundant_group_singleton",
                format!("redundant_group '{group}' has only one member"),
                None,
            );
            continue;
        }
        let mut priorities: Vec<u32> = members.iter().filter_map(|d| d.failover_priority).collect();
        priorities.sort();
        priorities.dedup();
        if priorities.len() != members.len() {
            report.push(
                ValidationSeverity::Error,
                "device.redundant_missing_priority",
                format!(
                    "redundant_group '{group}' members must each declare unique failover_priority"
                ),
                None,
            );
        }
    }

    report
}
