use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ModuleType {
    Oscillator,
    Sequencer,
    Filter,
    Reverb,
    Delay,
    Output,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleParameter {
    pub name: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
}

impl ModuleParameter {
    pub fn new(name: &str, value: f32, min: f32, max: f32) -> Self {
        Self {
            name: name.to_string(),
            value,
            min,
            max,
            step: (max - min) / 100.0, // Default to 100 steps
        }
    }

    pub fn adjust(&mut self, delta: f32) {
        self.value = (self.value + delta * (self.max - self.min)).clamp(self.min, self.max);
    }

    pub fn normalized(&self) -> f32 {
        (self.value - self.min) / (self.max - self.min)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from_module: Uuid,
    pub from_output: usize,
    pub to_module: Uuid,
    pub to_input: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioModule {
    pub id: Uuid,
    pub module_type: ModuleType,
    pub name: String,
    pub parameters: Vec<ModuleParameter>,
    pub input_count: usize,
    pub output_count: usize,
    pub position: (u16, u16),
    pub enabled: bool,
}

impl AudioModule {
    pub fn new(module_type: ModuleType, name: &str) -> Self {
        let (input_count, output_count, parameters) = match module_type {
            ModuleType::Oscillator => (0, 1, Self::oscillator_params()),
            ModuleType::Filter => (1, 1, Self::filter_params()),
            ModuleType::Reverb => (1, 1, Self::reverb_params()),
            ModuleType::Delay => (1, 1, Self::delay_params()),
            ModuleType::Output => (2, 0, Self::output_params()),
            ModuleType::Sequencer => (0, 1, Self::sequencer_params()),
        };

        Self {
            id: Uuid::new_v4(),
            module_type,
            name: name.to_string(),
            parameters,
            input_count,
            output_count,
            position: (0, 0),
            enabled: true,
        }
    }

    fn oscillator_params() -> Vec<ModuleParameter> {
        vec![
            ModuleParameter::new("Frequency", 440.0, 20.0, 20000.0),
            ModuleParameter::new("Amplitude", 0.5, 0.0, 1.0),
            ModuleParameter::new("Waveform", 0.0, 0.0, 3.0), // 0=sine, 1=saw, 2=square, 3=triangle
        ]
    }

    fn filter_params() -> Vec<ModuleParameter> {
        vec![
            ModuleParameter::new("Cutoff", 1000.0, 20.0, 20000.0),
            ModuleParameter::new("Resonance", 0.5, 0.0, 1.0),
            ModuleParameter::new("Type", 0.0, 0.0, 2.0), // 0=lowpass, 1=highpass, 2=bandpass
        ]
    }

    fn reverb_params() -> Vec<ModuleParameter> {
        vec![
            ModuleParameter::new("Room Size", 0.5, 0.0, 1.0),
            ModuleParameter::new("Damping", 0.5, 0.0, 1.0),
            ModuleParameter::new("Wet", 0.3, 0.0, 1.0),
        ]
    }

    fn delay_params() -> Vec<ModuleParameter> {
        vec![
            ModuleParameter::new("Time", 0.25, 0.01, 2.0),
            ModuleParameter::new("Feedback", 0.3, 0.0, 0.95),
            ModuleParameter::new("Wet", 0.3, 0.0, 1.0),
        ]
    }

    fn output_params() -> Vec<ModuleParameter> {
        vec![
            ModuleParameter::new("Volume", 0.7, 0.0, 1.0),
            ModuleParameter::new("Pan", 0.5, 0.0, 1.0),
        ]
    }

    fn sequencer_params() -> Vec<ModuleParameter> {
        vec![
            ModuleParameter::new("BPM", 120.0, 60.0, 200.0),
            ModuleParameter::new("Steps", 16.0, 4.0, 32.0),
            ModuleParameter::new("Gate", 0.5, 0.1, 1.0),
        ]
    }

    pub fn get_parameter(&self, name: &str) -> Option<&ModuleParameter> {
        self.parameters.iter().find(|p| p.name == name)
    }

    pub fn get_parameter_mut(&mut self, name: &str) -> Option<&mut ModuleParameter> {
        self.parameters.iter_mut().find(|p| p.name == name)
    }

    pub fn get_parameter_value(&self, name: &str) -> f32 {
        self.get_parameter(name).map(|p| p.value).unwrap_or(0.0)
    }
}
