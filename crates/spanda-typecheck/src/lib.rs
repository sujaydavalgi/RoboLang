//! Spanda physical units and compile-time type system primitives.
//!
pub mod checker;
pub mod diagnostics;
pub mod host;
pub mod message_registry;
pub mod module_registry;
pub mod type_system;
pub mod units;

pub use checker::{
    check, check_with_registry, format_type_name, get_library_for_sensor_type, merge_library_methods,
    type_check, units_compatible, MethodSig, TypeCheckError, TypeChecker, ACTUATOR_TYPES,
    AI_MODEL_TYPES, AI_VALUE_TYPES, BUILTIN_FUNCTIONS, BUILTIN_METHODS, MESSAGE_TYPES,
    OBJECT_PROPERTIES, POSE_PROPERTIES, ROBOT_METHODS, SCAN_PROPERTIES, SENSOR_TYPES,
    SERVICE_TYPES, ACTION_TYPES, VELOCITY_PROPERTIES,
};
pub use diagnostics::Diagnostic;
pub use host::TypeCheckHost;
pub use message_registry::{is_comm_capability, MessageRegistry, COMM_CAPABILITIES};
pub use module_registry::{ModuleExports, ModuleRegistry};
