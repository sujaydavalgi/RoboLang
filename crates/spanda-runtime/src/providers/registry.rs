//! Runtime registry for optional domain provider implementations.
//!
use super::hri::{
    HriInputProvider, OverlayProvider, SpatialSessionProvider, WearableTelemetryProvider,
};
use super::traits::{
    ActuatorProvider, CloudProvider, ConnectivityProvider, CryptoProvider, FleetProvider,
    HalProvider, LedgerProvider, MaintenanceProvider, NavigationProvider, PositioningProvider,
    RosProvider, SensorProvider, SimulationProvider, SlamProvider, TransportProvider,
    VisionProvider,
};
use super::types::{ProviderCapabilitySet, ProviderId};
use spanda_ast::comm_decl::TransportKind;
use std::collections::HashMap;

/// Holds installed provider implementations keyed by package and name.
#[derive(Default)]
pub struct ProviderRegistry {
    sensors: HashMap<String, Box<dyn SensorProvider>>,
    actuators: HashMap<String, Box<dyn ActuatorProvider>>,
    connectivity: HashMap<String, Box<dyn ConnectivityProvider>>,
    positioning: HashMap<String, Box<dyn PositioningProvider>>,
    transports: HashMap<String, Box<dyn TransportProvider>>,
    crypto: HashMap<String, Box<dyn CryptoProvider>>,
    navigation: HashMap<String, Box<dyn NavigationProvider>>,
    slam: HashMap<String, Box<dyn SlamProvider>>,
    vision: HashMap<String, Box<dyn VisionProvider>>,
    fleet: HashMap<String, Box<dyn FleetProvider>>,
    simulation: HashMap<String, Box<dyn SimulationProvider>>,
    maintenance: HashMap<String, Box<dyn MaintenanceProvider>>,
    ledger: HashMap<String, Box<dyn LedgerProvider>>,
    cloud: HashMap<String, Box<dyn CloudProvider>>,
    ros: HashMap<String, Box<dyn RosProvider>>,
    hal: HashMap<String, Box<dyn HalProvider>>,
    wearable_telemetry: HashMap<String, Box<dyn WearableTelemetryProvider>>,
    spatial_sessions: HashMap<String, Box<dyn SpatialSessionProvider>>,
    hri_inputs: HashMap<String, Box<dyn HriInputProvider>>,
    overlays: HashMap<String, Box<dyn OverlayProvider>>,
    granted_capabilities: ProviderCapabilitySet,
    official_packages: Vec<String>,
}

fn registry_key(id: &ProviderId) -> String {
    // Description:

    //     Registry key.

    //

    // Inputs:

    //     id: &ProviderId

    //         Caller-supplied id.

    //

    // Outputs:

    //     result: String

    //         Return value from `registry_key`.

    //

    // Example:

    //     let result = spanda_runtime::registry::registry_key(id);

    format!("{}::{}", id.package, id.name)
}

/// Stable registry lookup key for a project-scoped transport provider.
pub fn transport_registry_key(package: &str) -> String {
    // Description:
    //     Transport registry key.
    //
    // Inputs:
    //     package: &str
    //         Caller-supplied package.
    //
    // Outputs:
    //     result: String
    //         Return value from `transport_registry_key`.
    //
    // Example:
    //     let result = spanda_runtime::registry::transport_registry_key(package);

    format!("{package}::project")
}

impl ProviderRegistry {
    pub fn new() -> Self {
        // Description:
        //     Construct a new instance.
        //
        // Inputs:
        //     None.
        //
        // Outputs:
        //     result: Self
        //         Return value from `new`.
        //
        // Example:
        //     let value = spanda_runtime::registry::new();

        Self::default()
    }

    pub fn grant_capability(&mut self, cap: impl Into<String>) {
        // Description:

        //     Grant capability.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     cap: impl Into<String>

        //         Caller-supplied cap.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_runtime::registry::grant_capability(&mut self, cap);

        self.granted_capabilities.insert(cap);
    }

    pub fn has_capability(&self, cap: &str) -> bool {
        // Description:

        //     Has capability.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //     cap: &str

        //         Caller-supplied cap.

        //

        // Outputs:

        //     result: bool

        //         Return value from `has_capability`.

        //

        // Example:

        //     let result = spanda_runtime::registry::has_capability(&self, cap);

        self.granted_capabilities.contains(cap)
    }

    pub fn set_official_packages(&mut self, names: Vec<String>) {
        // Description:

        //     Set official packages.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     names: Vec<String>

        //         Caller-supplied names.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_runtime::registry::set_official_packages(&mut self, names);

        self.official_packages = names;
    }

    pub fn official_packages(&self) -> &[String] {
        // Description:

        //     Official packages.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: &[String]

        //         Return value from `official_packages`.

        //

        // Example:

        //     let result = spanda_runtime::registry::official_packages(&self);

        &self.official_packages
    }

    pub fn has_official_package(&self, name: &str) -> bool {
        // Description:

        //     Has official package.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //     name: &str

        //         Caller-supplied name.

        //

        // Outputs:

        //     result: bool

        //         Return value from `has_official_package`.

        //

        // Example:

        //     let result = spanda_runtime::registry::has_official_package(&self, name);

        self.official_packages.iter().any(|pkg| pkg == name)
    }

    pub fn register_sensor(&mut self, provider: Box<dyn SensorProvider>) {
        // Description:

        //     Register sensor.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     provider: Box<dyn SensorProvider>

        //         Caller-supplied provider.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_runtime::registry::register_sensor(&mut self, provider);

        let key = registry_key(&provider.metadata().id);
        self.sensors.insert(key, provider);
    }

    pub fn register_actuator(&mut self, provider: Box<dyn ActuatorProvider>) {
        // Description:

        //     Register actuator.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     provider: Box<dyn ActuatorProvider>

        //         Caller-supplied provider.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_runtime::registry::register_actuator(&mut self, provider);

        let key = registry_key(&provider.metadata().id);
        self.actuators.insert(key, provider);
    }

    pub fn register_connectivity(&mut self, provider: Box<dyn ConnectivityProvider>) {
        // Description:

        //     Register connectivity.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     provider: Box<dyn ConnectivityProvider>

        //         Caller-supplied provider.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_runtime::registry::register_connectivity(&mut self, provider);

        let key = registry_key(&provider.metadata().id);
        self.connectivity.insert(key, provider);
    }

    pub fn register_positioning(&mut self, provider: Box<dyn PositioningProvider>) {
        // Description:

        //     Register positioning.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     provider: Box<dyn PositioningProvider>

        //         Caller-supplied provider.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_runtime::registry::register_positioning(&mut self, provider);

        let key = registry_key(&provider.metadata().id);
        self.positioning.insert(key, provider);
    }

    pub fn register_transport(&mut self, provider: Box<dyn TransportProvider>) {
        // Description:

        //     Register transport.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     provider: Box<dyn TransportProvider>

        //         Caller-supplied provider.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_runtime::registry::register_transport(&mut self, provider);

        let key = registry_key(&provider.metadata().id);
        self.transports.insert(key, provider);
    }

    pub fn register_crypto(&mut self, provider: Box<dyn CryptoProvider>) {
        // Description:

        //     Register crypto.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     provider: Box<dyn CryptoProvider>

        //         Caller-supplied provider.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_runtime::registry::register_crypto(&mut self, provider);

        let key = registry_key(&provider.metadata().id);
        self.crypto.insert(key, provider);
    }

    pub fn register_navigation(&mut self, provider: Box<dyn NavigationProvider>) {
        // Description:

        //     Register navigation.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     provider: Box<dyn NavigationProvider>

        //         Caller-supplied provider.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_runtime::registry::register_navigation(&mut self, provider);

        let key = registry_key(&provider.metadata().id);
        self.navigation.insert(key, provider);
    }

    pub fn register_slam(&mut self, provider: Box<dyn SlamProvider>) {
        // Description:

        //     Register slam.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     provider: Box<dyn SlamProvider>

        //         Caller-supplied provider.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_runtime::registry::register_slam(&mut self, provider);

        let key = registry_key(&provider.metadata().id);
        self.slam.insert(key, provider);
    }

    pub fn register_vision(&mut self, provider: Box<dyn VisionProvider>) {
        // Description:

        //     Register vision.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     provider: Box<dyn VisionProvider>

        //         Caller-supplied provider.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_runtime::registry::register_vision(&mut self, provider);

        let key = registry_key(&provider.metadata().id);
        self.vision.insert(key, provider);
    }

    pub fn register_fleet(&mut self, provider: Box<dyn FleetProvider>) {
        // Description:

        //     Register fleet.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     provider: Box<dyn FleetProvider>

        //         Caller-supplied provider.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_runtime::registry::register_fleet(&mut self, provider);

        let key = registry_key(&provider.metadata().id);
        self.fleet.insert(key, provider);
    }

    pub fn register_simulation(&mut self, provider: Box<dyn SimulationProvider>) {
        // Description:

        //     Register simulation.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     provider: Box<dyn SimulationProvider>

        //         Caller-supplied provider.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_runtime::registry::register_simulation(&mut self, provider);

        let key = registry_key(&provider.metadata().id);
        self.simulation.insert(key, provider);
    }

    pub fn register_maintenance(&mut self, provider: Box<dyn MaintenanceProvider>) {
        // Description:

        //     Register maintenance.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     provider: Box<dyn MaintenanceProvider>

        //         Caller-supplied provider.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_runtime::registry::register_maintenance(&mut self, provider);

        let key = registry_key(&provider.metadata().id);
        self.maintenance.insert(key, provider);
    }

    pub fn register_ledger(&mut self, provider: Box<dyn LedgerProvider>) {
        // Description:

        //     Register ledger.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     provider: Box<dyn LedgerProvider>

        //         Caller-supplied provider.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_runtime::registry::register_ledger(&mut self, provider);

        let key = registry_key(&provider.metadata().id);
        self.ledger.insert(key, provider);
    }

    pub fn register_cloud(&mut self, provider: Box<dyn CloudProvider>) {
        // Description:

        //     Register cloud.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     provider: Box<dyn CloudProvider>

        //         Caller-supplied provider.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_runtime::registry::register_cloud(&mut self, provider);

        let key = registry_key(&provider.metadata().id);
        self.cloud.insert(key, provider);
    }

    pub fn register_ros(&mut self, provider: Box<dyn RosProvider>) {
        // Description:

        //     Register ros.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     provider: Box<dyn RosProvider>

        //         Caller-supplied provider.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_runtime::registry::register_ros(&mut self, provider);

        let key = registry_key(&provider.metadata().id);
        self.ros.insert(key, provider);
    }

    pub fn register_hal(&mut self, provider: Box<dyn HalProvider>) {
        // Description:

        //     Register hal.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     provider: Box<dyn HalProvider>

        //         Caller-supplied provider.

        //

        // Outputs:

        //     None.

        //

        // Example:

        //     let result = spanda_runtime::registry::register_hal(&mut self, provider);

        let key = registry_key(&provider.metadata().id);
        self.hal.insert(key, provider);
    }

    pub fn list_transports(&self) -> Vec<ProviderId> {
        // Description:

        //     List transports.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: Vec<ProviderId>

        //         Return value from `list_transports`.

        //

        // Example:

        //     let result = spanda_runtime::registry::list_transports(&self);

        self.transports
            .values()
            .map(|p| p.metadata().id.clone())
            .collect()
    }

    pub fn list_positioning(&self) -> Vec<ProviderId> {
        // Description:

        //     List positioning.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: Vec<ProviderId>

        //         Return value from `list_positioning`.

        //

        // Example:

        //     let result = spanda_runtime::registry::list_positioning(&self);

        self.positioning
            .values()
            .map(|p| p.metadata().id.clone())
            .collect()
    }

    pub fn list_fleet(&self) -> Vec<ProviderId> {
        // Description:

        //     List fleet.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: Vec<ProviderId>

        //         Return value from `list_fleet`.

        //

        // Example:

        //     let result = spanda_runtime::registry::list_fleet(&self);

        self.fleet
            .values()
            .map(|p| p.metadata().id.clone())
            .collect()
    }

    pub fn transport_count(&self) -> usize {
        // Description:

        //     Transport count.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: usize

        //         Return value from `transport_count`.

        //

        // Example:

        //     let result = spanda_runtime::registry::transport_count(&self);

        self.transports.len()
    }

    pub fn with_transport<F, R>(&mut self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn TransportProvider) -> R,
    {
        // Description:

        //     With transport.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     key: &str

        //         Caller-supplied key.

        //     f: F

        //         Caller-supplied f.

        //

        // Outputs:

        //     result: Option<R> where F: FnOnce(&mut dyn TransportProvider) -> R,

        //         Return value from `with_transport`.

        //

        // Example:

        //     let result = spanda_runtime::registry::with_transport(&mut self, key, f);

        self.transports.get_mut(key).map(|p| f(p.as_mut()))
    }

    pub fn with_transport_for_kind<F, R>(&mut self, kind: TransportKind, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn TransportProvider) -> R,
    {
        // Description:

        //     With transport for kind.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     kind: TransportKind

        //         Caller-supplied kind.

        //     f: F

        //         Caller-supplied f.

        //

        // Outputs:

        //     result: Option<R> where F: FnOnce(&mut dyn TransportProvider) -> R,

        //         Return value from `with_transport_for_kind`.

        //

        // Example:

        //     let result = spanda_runtime::registry::with_transport_for_kind(&mut self, kind, f);

        let key = self
            .transports
            .iter()
            .find(|(_, provider)| provider.kind() == kind)
            .map(|(key, _)| key.clone())?;
        self.with_transport(&key, f)
    }

    pub fn with_positioning<F, R>(&mut self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn PositioningProvider) -> R,
    {
        // Description:

        //     With positioning.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     key: &str

        //         Caller-supplied key.

        //     f: F

        //         Caller-supplied f.

        //

        // Outputs:

        //     result: Option<R> where F: FnOnce(&mut dyn PositioningProvider) -> R,

        //         Return value from `with_positioning`.

        //

        // Example:

        //     let result = spanda_runtime::registry::with_positioning(&mut self, key, f);

        self.positioning.get_mut(key).map(|p| f(p.as_mut()))
    }

    pub fn with_connectivity<F, R>(&mut self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn ConnectivityProvider) -> R,
    {
        // Description:

        //     With connectivity.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     key: &str

        //         Caller-supplied key.

        //     f: F

        //         Caller-supplied f.

        //

        // Outputs:

        //     result: Option<R> where F: FnOnce(&mut dyn ConnectivityProvider) -> R,

        //         Return value from `with_connectivity`.

        //

        // Example:

        //     let result = spanda_runtime::registry::with_connectivity(&mut self, key, f);

        self.connectivity.get_mut(key).map(|p| f(p.as_mut()))
    }

    pub fn with_navigation<F, R>(&mut self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn NavigationProvider) -> R,
    {
        // Description:

        //     With navigation.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     key: &str

        //         Caller-supplied key.

        //     f: F

        //         Caller-supplied f.

        //

        // Outputs:

        //     result: Option<R> where F: FnOnce(&mut dyn NavigationProvider) -> R,

        //         Return value from `with_navigation`.

        //

        // Example:

        //     let result = spanda_runtime::registry::with_navigation(&mut self, key, f);

        self.navigation.get_mut(key).map(|p| f(p.as_mut()))
    }

    pub fn with_fleet<F, R>(&mut self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn FleetProvider) -> R,
    {
        // Description:

        //     With fleet.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     key: &str

        //         Caller-supplied key.

        //     f: F

        //         Caller-supplied f.

        //

        // Outputs:

        //     result: Option<R> where F: FnOnce(&mut dyn FleetProvider) -> R,

        //         Return value from `with_fleet`.

        //

        // Example:

        //     let result = spanda_runtime::registry::with_fleet(&mut self, key, f);

        self.fleet.get_mut(key).map(|p| f(p.as_mut()))
    }

    pub fn with_slam<F, R>(&mut self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn SlamProvider) -> R,
    {
        // Description:

        //     With slam.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     key: &str

        //         Caller-supplied key.

        //     f: F

        //         Caller-supplied f.

        //

        // Outputs:

        //     result: Option<R> where F: FnOnce(&mut dyn SlamProvider) -> R,

        //         Return value from `with_slam`.

        //

        // Example:

        //     let result = spanda_runtime::registry::with_slam(&mut self, key, f);

        self.slam.get_mut(key).map(|p| f(p.as_mut()))
    }

    pub fn with_vision<F, R>(&mut self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn VisionProvider) -> R,
    {
        // Description:

        //     With vision.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     key: &str

        //         Caller-supplied key.

        //     f: F

        //         Caller-supplied f.

        //

        // Outputs:

        //     result: Option<R> where F: FnOnce(&mut dyn VisionProvider) -> R,

        //         Return value from `with_vision`.

        //

        // Example:

        //     let result = spanda_runtime::registry::with_vision(&mut self, key, f);

        self.vision.get_mut(key).map(|p| f(p.as_mut()))
    }

    pub fn positioning_count(&self) -> usize {
        // Description:

        //     Positioning count.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: usize

        //         Return value from `positioning_count`.

        //

        // Example:

        //     let result = spanda_runtime::registry::positioning_count(&self);

        self.positioning.len()
    }

    pub fn connectivity_count(&self) -> usize {
        // Description:

        //     Connectivity count.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: usize

        //         Return value from `connectivity_count`.

        //

        // Example:

        //     let result = spanda_runtime::registry::connectivity_count(&self);

        self.connectivity.len()
    }

    pub fn navigation_count(&self) -> usize {
        // Description:

        //     Navigation count.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: usize

        //         Return value from `navigation_count`.

        //

        // Example:

        //     let result = spanda_runtime::registry::navigation_count(&self);

        self.navigation.len()
    }

    pub fn fleet_count(&self) -> usize {
        // Description:

        //     Fleet count.

        //

        // Inputs:

        //     &self: value

        //         Caller-supplied &self.

        //

        // Outputs:

        //     result: usize

        //         Return value from `fleet_count`.

        //

        // Example:

        //     let result = spanda_runtime::registry::fleet_count(&self);

        self.fleet.len()
    }

    pub fn with_simulation<F, R>(&mut self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn SimulationProvider) -> R,
    {
        // Description:

        //     With simulation.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     key: &str

        //         Caller-supplied key.

        //     f: F

        //         Caller-supplied f.

        //

        // Outputs:

        //     result: Option<R> where F: FnOnce(&mut dyn SimulationProvider) -> R,

        //         Return value from `with_simulation`.

        //

        // Example:

        //     let result = spanda_runtime::registry::with_simulation(&mut self, key, f);

        self.simulation.get_mut(key).map(|p| f(p.as_mut()))
    }

    pub fn with_ledger<F, R>(&mut self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn LedgerProvider) -> R,
    {
        // Description:

        //     With ledger.

        //

        // Inputs:

        //     &mut self: value

        //         Caller-supplied &mut self.

        //     key: &str

        //         Caller-supplied key.

        //     f: F

        //         Caller-supplied f.

        //

        // Outputs:

        //     result: Option<R> where F: FnOnce(&mut dyn LedgerProvider) -> R,

        //         Return value from `with_ledger`.

        //

        // Example:

        //     let result = spanda_runtime::registry::with_ledger(&mut self, key, f);

        self.ledger.get_mut(key).map(|p| f(p.as_mut()))
    }

    pub fn register_wearable_telemetry(&mut self, provider: Box<dyn WearableTelemetryProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.wearable_telemetry.insert(key, provider);
    }

    pub fn register_spatial_session(&mut self, provider: Box<dyn SpatialSessionProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.spatial_sessions.insert(key, provider);
    }

    pub fn with_wearable_telemetry<F, R>(&mut self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn WearableTelemetryProvider) -> R,
    {
        self.wearable_telemetry.get_mut(key).map(|p| f(p.as_mut()))
    }

    pub fn with_spatial_session<F, R>(&mut self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn SpatialSessionProvider) -> R,
    {
        self.spatial_sessions.get_mut(key).map(|p| f(p.as_mut()))
    }

    pub fn register_hri_input(&mut self, provider: Box<dyn HriInputProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.hri_inputs.insert(key, provider);
    }

    pub fn register_overlay(&mut self, provider: Box<dyn OverlayProvider>) {
        let key = registry_key(&provider.metadata().id);
        self.overlays.insert(key, provider);
    }

    pub fn with_hri_input<F, R>(&mut self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn HriInputProvider) -> R,
    {
        self.hri_inputs.get_mut(key).map(|p| f(p.as_mut()))
    }

    pub fn with_overlay<F, R>(&mut self, key: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut dyn OverlayProvider) -> R,
    {
        self.overlays.get_mut(key).map(|p| f(p.as_mut()))
    }
}
