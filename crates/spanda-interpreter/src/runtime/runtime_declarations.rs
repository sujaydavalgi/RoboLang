//! Topic, service, sensor, and actuator declaration helpers.
//!

use super::{Interpreter, RobotBackend, RuntimeValue};
use crate::ai::{create_agent_runtime, MemoryStore};
use spanda_ast::nodes::{
    ActionDecl, ActuatorDecl, AgentDecl, SensorBinding, SensorDecl, ServiceDecl, TopicDecl,
};
use spanda_comm::CommBus;

impl<B: RobotBackend> Interpreter<B> {
    pub(super) fn define_topic(&mut self, topic: &TopicDecl) {
        // Define topic.
        //
        // Parameters:
        // - `self` — method receiver
        // - `topic` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.define_topic(topic);

        // Compute TopicDecl for the following logic.
        let TopicDecl::TopicDecl {
            name,
            message_type,
            topic: topic_path,
            transport,
            secure,
            qos,
            ..
        } = topic;
        let path = topic_path.clone().unwrap_or_else(|| format!("/{name}"));
        let transport = transport.unwrap_or(self.default_transport);

        // Emit output when secure provides a block.
        if let Some(block) = secure {
            self.security
                .register_secure_endpoint(&path, Self::secure_policy_from_block(block));
        }
        self.comm_bus.subscribe(&path, name);
        self.topic_path_to_name.insert(path.clone(), name.clone());
        self.topic_path_to_message_type
            .insert(path.clone(), message_type.clone());
        if let Some(qos_decl) = qos {
            self.topic_qos.insert(path.clone(), qos_decl.clone());
        }
        self.env.define(
            name.clone(),
            RuntimeValue::Topic {
                name: name.clone(),
                message_type: message_type.clone(),
                topic_path: path,
            },
        );
        let _ = transport;
    }

    pub(super) fn define_service(&mut self, service: &ServiceDecl) {
        // Define service.
        //
        // Parameters:
        // - `self` — method receiver
        // - `service` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.define_service(service);

        // Compute ServiceDecl for the following logic.
        let ServiceDecl::ServiceDecl {
            name,
            service_type,
            request_type,
            response_type,
            secure,
            ..
        } = service;
        let endpoint = format!("/service/{name}");

        // Emit output when secure provides a block.
        if let Some(block) = secure {
            self.security
                .register_secure_endpoint(&endpoint, Self::secure_policy_from_block(block));
        }
        let st = service_type
            .clone()
            .or_else(|| response_type.clone())
            .unwrap_or_else(|| name.clone());
        self.env.define(
            name.clone(),
            RuntimeValue::Service {
                name: name.clone(),
                service_type: st,
            },
        );
        let _ = request_type;
    }

    pub(super) fn define_action(&mut self, action: &ActionDecl) {
        // Define action.
        //
        // Parameters:
        // - `self` — method receiver
        // - `action` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.define_action(action);

        // Compute ActionDecl for the following logic.
        let ActionDecl::ActionDecl {
            name,
            action_type,
            result_type,
            secure,
            ..
        } = action;
        let endpoint = format!("/action/{name}");

        // Emit output when secure provides a block.
        if let Some(block) = secure {
            self.security
                .register_secure_endpoint(&endpoint, Self::secure_policy_from_block(block));
        }
        let at = action_type
            .clone()
            .or_else(|| result_type.clone())
            .unwrap_or_else(|| name.clone());
        self.env.define(
            name.clone(),
            RuntimeValue::Action {
                name: name.clone(),
                action_type: at,
            },
        );
    }

    pub(super) fn define_sensor(&mut self, sensor: &SensorDecl) {
        // Define sensor.
        //
        // Parameters:
        // - `self` — method receiver
        // - `sensor` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.define_sensor(sensor);

        // Compute SensorDecl for the following logic.
        let SensorDecl::SensorDecl {
            name,
            sensor_type,
            library,
            binding,
            ..
        } = sensor;
        let (topic, hal_binding) = match binding {
            Some(SensorBinding::Topic { path }) => (Some(path.clone()), None),
            Some(SensorBinding::Hal { bus_name }) => (None, Some(bus_name.clone())),
            None => (None, None),
        };
        self.env.define(
            name.clone(),
            RuntimeValue::Sensor {
                name: name.clone(),
                sensor_type: sensor_type.clone(),
                library: library.clone(),
                hal_binding,
                topic,
            },
        );
        self.hardware_monitor
            .register_sensor(name.clone(), sensor_type.clone());
    }

    pub(super) fn define_actuator(&mut self, actuator: &ActuatorDecl) {
        // Define actuator.
        //
        // Parameters:
        // - `self` — method receiver
        // - `actuator` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.define_actuator(actuator);

        // Compute ActuatorDecl for the following logic.
        let ActuatorDecl::ActuatorDecl {
            name,
            actuator_type,
            ..
        } = actuator;
        self.hardware_monitor
            .register_actuator(name.clone(), actuator_type.clone());
        self.env.define(
            name.clone(),
            RuntimeValue::Actuator {
                name: name.clone(),
                actuator_type: actuator_type.clone(),
            },
        );
    }

    pub(super) fn setup_agent(&mut self, agent_decl: &AgentDecl) {
        // Setup agent.
        //
        // Parameters:
        // - `self` — method receiver
        // - `agent_decl` — input value
        //
        // Returns:
        // Nothing.
        //
        // Options:
        // None.
        //
        // Example:
        // let result = instance.setup_agent(agent_decl);

        // Compute AgentDecl for the following logic.
        let AgentDecl::AgentDecl {
            name,
            goal,
            memory_kind,
            capabilities,
            ..
        } = agent_decl;
        let memory = memory_kind.map(|k| MemoryStore::new(k.into(), None));
        let agent = create_agent_runtime(agent_decl.clone(), memory);
        self.agents.insert(name.clone(), agent);
        self.agent_capabilities
            .insert(name.clone(), capabilities.clone());
        self.comm_bus.register_agent(name);
        self.env
            .define(name.clone(), RuntimeValue::Agent { name: name.clone() });
        self.log(format!("Agent '{name}': {goal}"));
    }

}
