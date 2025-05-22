use anyhow::Result;
use fundsp::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

use crate::modules::{AudioModule, Connection, ModuleType};

/// Represents a node in our audio graph
pub struct AudioGraphNode {
    pub module_id: Uuid,
    pub audio_unit: Box<dyn AudioUnit64>,
    pub input_connections: Vec<Option<Uuid>>,
    pub output_connections: Vec<Uuid>,
}

/// Manages the entire audio processing graph
pub struct AudioGraph {
    nodes: HashMap<Uuid, AudioGraphNode>,
    connections: Vec<Connection>,
    sample_rate: f64,
    output_node: Option<Uuid>,
}

impl AudioGraph {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
            sample_rate,
            output_node: None,
        }
    }

    /// Add a module to the audio graph
    pub fn add_module(&mut self, module: &AudioModule) -> Result<()> {
        let audio_unit = self.create_audio_unit(module)?;

        let node = AudioGraphNode {
            module_id: module.id,
            audio_unit,
            input_connections: vec![None; module.input_count],
            output_connections: Vec::new(),
        };

        self.nodes.insert(module.id, node);

        // If this is an output module, set it as our output
        if module.module_type == ModuleType::Output {
            self.output_node = Some(module.id);
        }

        Ok(())
    }

    /// Remove a module from the audio graph
    pub fn remove_module(&mut self, module_id: Uuid) {
        // Remove all connections involving this module
        self.connections
            .retain(|conn| conn.from_module != module_id && conn.to_module != module_id);

        // Remove the node
        self.nodes.remove(&module_id);

        // Clear output node if it was removed
        if self.output_node == Some(module_id) {
            self.output_node = None;
        }

        self.rebuild_connections();
    }

    /// Add a connection between two modules
    pub fn add_connection(&mut self, connection: Connection) -> Result<()> {
        // Validate connection
        if !self.nodes.contains_key(&connection.from_module) {
            return Err(anyhow::anyhow!("Source module not found"));
        }
        if !self.nodes.contains_key(&connection.to_module) {
            return Err(anyhow::anyhow!("Destination module not found"));
        }

        // Check for cycles (basic check - could be more sophisticated)
        if self.would_create_cycle(&connection) {
            return Err(anyhow::anyhow!("Connection would create a cycle"));
        }

        self.connections.push(connection);
        self.rebuild_connections();
        Ok(())
    }

    /// Remove a connection
    pub fn remove_connection(
        &mut self,
        from_module: Uuid,
        from_output: usize,
        to_module: Uuid,
        to_input: usize,
    ) {
        self.connections.retain(|conn| {
            !(conn.from_module == from_module
                && conn.from_output == from_output
                && conn.to_module == to_module
                && conn.to_input == to_input)
        });
        self.rebuild_connections();
    }

    /// Update a module's parameters
    pub fn update_module(&mut self, module: &AudioModule) -> Result<()> {
        if let Some(node) = self.nodes.get_mut(&module.id) {
            // Recreate the audio unit with new parameters
            node.audio_unit = self.create_audio_unit(module)?;
        }
        Ok(())
    }

    /// Process audio through the graph
    pub fn process(&mut self, output: &mut [f64]) -> Result<()> {
        // For now, simple processing - we'll make this more sophisticated later
        if let Some(output_id) = self.output_node {
            if let Some(output_node) = self.nodes.get_mut(&output_id) {
                let mut buffer = vec![0.0; output.len()];

                // Process the output node
                // Note: This is a simplified version - real implementation would
                // need proper graph traversal and buffer management
                for (i, sample) in buffer.iter_mut().enumerate() {
                    *sample = output_node.audio_unit.tick(&[0.0, 0.0])[0];
                }

                output.copy_from_slice(&buffer);
            }
        } else {
            // No output node, fill with silence
            output.fill(0.0);
        }

        Ok(())
    }

    /// Create an audio unit from a module
    fn create_audio_unit(&self, module: &AudioModule) -> Result<Box<dyn AudioUnit64>> {
        match module.module_type {
            ModuleType::Oscillator => {
                let frequency = module.get_parameter_value("Frequency") as f64;
                let amplitude = module.get_parameter_value("Amplitude") as f64;
                let waveform = module.get_parameter_value("Waveform") as i32;

                let osc: Box<dyn AudioUnit64> = match waveform {
                    0 => Box::new(sine_hz(frequency) * amplitude),
                    1 => Box::new(saw_hz(frequency) * amplitude),
                    2 => Box::new(square_hz(frequency) * amplitude),
                    3 => Box::new(triangle_hz(frequency) * amplitude),
                    _ => Box::new(sine_hz(frequency) * amplitude),
                };

                Ok(osc)
            }
            ModuleType::Filter => {
                let cutoff = module.get_parameter_value("Cutoff") as f64;
                let resonance = module.get_parameter_value("Resonance") as f64;
                let filter_type = module.get_parameter_value("Type") as i32;

                let filter: Box<dyn AudioUnit64> = match filter_type {
                    0 => Box::new(lowpass_hz(cutoff, resonance)),
                    1 => Box::new(highpass_hz(cutoff, resonance)),
                    2 => Box::new(bandpass_hz(cutoff, resonance)),
                    _ => Box::new(lowpass_hz(cutoff, resonance)),
                };

                Ok(filter)
            }
            ModuleType::Reverb => {
                let room_size = module.get_parameter_value("Room Size") as f64;
                let damping = module.get_parameter_value("Damping") as f64;
                let wet = module.get_parameter_value("Wet") as f64;

                // Simple reverb approximation
                let reverb: Box<dyn AudioUnit64> = Box::new(
                    pass() * (1.0 - wet)
                        + (delay(room_size * 0.1)
                            + delay(room_size * 0.13)
                            + delay(room_size * 0.17))
                            * (damping * 0.5)
                            * wet,
                );

                Ok(reverb)
            }
            ModuleType::Delay => {
                let time = module.get_parameter_value("Time") as f64;
                let feedback = module.get_parameter_value("Feedback") as f64;
                let wet = module.get_parameter_value("Wet") as f64;

                let delay_unit: Box<dyn AudioUnit64> =
                    Box::new(pass() * (1.0 - wet) + delay(time) * feedback * wet);

                Ok(delay_unit)
            }
            ModuleType::Output => {
                let volume = module.get_parameter_value("Volume") as f64;
                let pan = module.get_parameter_value("Pan") as f64;

                // Simple stereo output
                let output: Box<dyn AudioUnit64> =
                    Box::new((pass() * volume * (1.0 - pan)) | (pass() * volume * pan));

                Ok(output)
            }
            ModuleType::Sequencer => {
                // TODO: Implement sequencer
                Ok(Box::new(dc(0.0)))
            }
        }
    }

    /// Rebuild internal connection structure
    fn rebuild_connections(&mut self) {
        // Clear existing connections
        for node in self.nodes.values_mut() {
            node.input_connections.fill(None);
            node.output_connections.clear();
        }

        // Rebuild from connection list
        for conn in &self.connections {
            if let Some(from_node) = self.nodes.get_mut(&conn.from_module) {
                if conn.from_output < from_node.output_connections.len() {
                    from_node.output_connections.push(conn.to_module);
                }
            }

            if let Some(to_node) = self.nodes.get_mut(&conn.to_module) {
                if conn.to_input < to_node.input_connections.len() {
                    to_node.input_connections[conn.to_input] = Some(conn.from_module);
                }
            }
        }
    }

    /// Check if adding a connection would create a cycle
    fn would_create_cycle(&self, new_connection: &Connection) -> bool {
        // Simple cycle detection - follow outputs from destination back to source
        fn has_path_to(
            graph: &AudioGraph,
            from: Uuid,
            target: Uuid,
            visited: &mut std::collections::HashSet<Uuid>,
        ) -> bool {
            if from == target {
                return true;
            }
            if visited.contains(&from) {
                return false;
            }
            visited.insert(from);

            if let Some(node) = graph.nodes.get(&from) {
                for &output_id in &node.output_connections {
                    if has_path_to(graph, output_id, target, visited) {
                        return true;
                    }
                }
            }
            false
        }

        let mut visited = std::collections::HashSet::new();
        has_path_to(
            self,
            new_connection.to_module,
            new_connection.from_module,
            &mut visited,
        )
    }

    /// Get all connections
    pub fn get_connections(&self) -> &[Connection] {
        &self.connections
    }

    /// Get modules that can be connected to a given input
    pub fn get_connectable_outputs(&self, to_module: Uuid, to_input: usize) -> Vec<(Uuid, usize)> {
        let mut result = Vec::new();

        for (module_id, node) in &self.nodes {
            if *module_id == to_module {
                continue; // Can't connect to self
            }

            // Check each output of this module
            for output_idx in 0..node.output_connections.len() {
                // Check if this would create a cycle
                let test_connection = Connection {
                    from_module: *module_id,
                    from_output: output_idx,
                    to_module,
                    to_input,
                };

                if !self.would_create_cycle(&test_connection) {
                    result.push((*module_id, output_idx));
                }
            }
        }

        result
    }
}
