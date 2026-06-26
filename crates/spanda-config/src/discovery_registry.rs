//! Registry-backed discovery package resolution at runtime.
//!
use crate::discovery_transport::{DeviceDiscoveryTransport, DiscoveryOptions, DiscoveryTransportResult};
use spanda_package::registry_package_dir;

/// Official discovery transport packages shipped in the registry.
const DISCOVERY_PACKAGES: &[(&str, &str)] = &[
    ("mdns", "spanda-discovery-mdns"),
    ("ble", "spanda-discovery-ble"),
    ("bluetooth", "spanda-discovery-ble"),
    ("usb", "spanda-discovery-usb"),
    ("wifi", "spanda-discovery-wifi"),
    ("cellular", "spanda-discovery-cellular"),
    ("lte", "spanda-discovery-cellular"),
    ("5g", "spanda-discovery-cellular"),
    ("serial", "spanda-discovery-serial"),
];

/// Map a transport name to its registry package when one exists.
pub fn discovery_package_for_transport(transport: &str) -> Option<&'static str> {
    let lower = transport.to_ascii_lowercase();
    DISCOVERY_PACKAGES
        .iter()
        .find(|(name, _)| *name == lower)
        .map(|(_, package)| *package)
}

/// True when the registry package source tree is present on disk.
pub fn is_registry_discovery_package_installed(package: &str) -> bool {
    registry_package_dir(package).is_some()
}

/// Installed discovery packages as `package:transport` labels.
pub fn list_installed_discovery_packages() -> Vec<String> {
    DISCOVERY_PACKAGES
        .iter()
        .filter(|(_, package)| is_registry_discovery_package_installed(package))
        .map(|(transport, package)| format!("{package}:{transport}"))
        .collect()
}

struct RegistryBackedDiscoveryTransport {
    inner: Box<dyn DeviceDiscoveryTransport>,
    package_name: &'static str,
}

impl DeviceDiscoveryTransport for RegistryBackedDiscoveryTransport {
    fn transport_name(&self) -> &'static str {
        self.inner.transport_name()
    }

    fn discover(&self, options: &DiscoveryOptions) -> Result<DiscoveryTransportResult, String> {
        let mut result = self.inner.discover(options)?;
        let label = format!("{}:{}", result.transport, self.package_name);
        result.transport = label.clone();
        for match_entry in &mut result.matches {
            match_entry.matched_by = label.clone();
        }
        Ok(result)
    }
}

/// Wrap a built-in transport with registry package metadata when the package is installed.
pub fn wrap_with_registry_package(
    transport: &str,
    inner: Box<dyn DeviceDiscoveryTransport>,
) -> Box<dyn DeviceDiscoveryTransport> {
    if let Some(package) = discovery_package_for_transport(transport) {
        if is_registry_discovery_package_installed(package) {
            return Box::new(RegistryBackedDiscoveryTransport {
                inner,
                package_name: package,
            });
        }
    }
    inner
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery_transport::{
        MockBleDiscoveryTransport, MockMdnsDiscoveryTransport, MockUsbDiscoveryTransport,
    };

    #[test]
    fn registry_mdns_wraps_when_package_present() {
        if !is_registry_discovery_package_installed("spanda-discovery-mdns") {
            return;
        }
        let transport = wrap_with_registry_package(
            "mdns",
            Box::new(MockMdnsDiscoveryTransport) as Box<dyn DeviceDiscoveryTransport>,
        );
        let result = transport.discover(&DiscoveryOptions::default()).unwrap();
        assert!(result.transport.contains("spanda-discovery-mdns"));
        assert!(result
            .matches
            .first()
            .is_some_and(|m| m.matched_by.contains("spanda-discovery-mdns")));
    }

    #[test]
    fn registry_ble_wraps_when_package_present() {
        if !is_registry_discovery_package_installed("spanda-discovery-ble") {
            return;
        }
        let transport = wrap_with_registry_package(
            "ble",
            Box::new(MockBleDiscoveryTransport) as Box<dyn DeviceDiscoveryTransport>,
        );
        let result = transport.discover(&DiscoveryOptions::default()).unwrap();
        assert!(result.transport.contains("spanda-discovery-ble"));
    }

    #[test]
    fn registry_usb_wraps_when_package_present() {
        if !is_registry_discovery_package_installed("spanda-discovery-usb") {
            return;
        }
        let transport = wrap_with_registry_package(
            "usb",
            Box::new(MockUsbDiscoveryTransport) as Box<dyn DeviceDiscoveryTransport>,
        );
        let result = transport.discover(&DiscoveryOptions::default()).unwrap();
        assert!(result.transport.contains("spanda-discovery-usb"));
    }
}
