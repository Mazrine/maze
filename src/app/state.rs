use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::Frame;
use std::collections::HashMap;
use uuid::Uuid;

use crate::app::modes::AppMode;
use crate::audio::AudioEngine;
use crate::modules::{AudioModule, Connection, ModuleType};
use crate::ui;

pub struct DAWApp {
    pub modules: HashMap<Uuid, AudioModule>,
    pub connections: Vec<Connection>,
    pub selected_module: Option<Uuid>,
    pub selected_parameter: usize,
    pub mode: AppMode,
    pub audio_engine: AudioEngine,
    pub _audio_stream: Option<cpal::Stream>, // Keep stream alive
}

impl DAWApp {
    pub fn new() -> Result<Self> {
        let audio_engine = AudioEngine::new()?;
        let audio_stream = audio_engine.start_audio_stream()?;

        // Start with a basic oscillator and output
        let mut modules = HashMap::new();
        let osc = AudioModule::new(ModuleType::Oscillator, "OSC-1");
        let output = AudioModule::new(ModuleType::Output, "MAIN-OUT");

        modules.insert(osc.id, osc);
        modules.insert(output.id, output);

        Ok(Self {
            modules,
            connections: Vec::new(),
            selected_module: None,
            selected_parameter: 0,
            mode: AppMode::ModuleView,
            audio_engine,
            _audio_stream: Some(audio_stream),
        })
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.mode.handle_key(self, key)
    }

    pub fn draw(&self, frame: &mut Frame) {
        ui::layout::draw_main_layout(self, frame);
    }

    pub fn cycle_selected_module(&mut self) {
        let module_ids: Vec<_> = self.modules.keys().cloned().collect();
        if !module_ids.is_empty() {
            if let Some(current) = self.selected_module {
                if let Some(pos) = module_ids.iter().position(|&id| id == current) {
                    let next_pos = (pos + 1) % module_ids.len();
                    self.selected_module = Some(module_ids[next_pos]);
                }
            } else {
                self.selected_module = Some(module_ids[0]);
            }
        }
    }

    pub fn adjust_selected_parameter(&mut self, delta: f32) {
        if let Some(module_id) = self.selected_module {
            if let Some(module) = self.modules.get_mut(&module_id) {
                if let Some(param) = module.parameters.get_mut(self.selected_parameter) {
                    param.adjust(delta);

                    // Update audio engine based on parameter changes
                    if module.module_type == ModuleType::Oscillator {
                        match param.name.as_str() {
                            "Frequency" => self.audio_engine.update_frequency(param.value),
                            "Amplitude" => self.audio_engine.update_amplitude(param.value),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    pub fn add_module(&mut self, module_type: ModuleType) {
        let name = match module_type {
            ModuleType::Oscillator => {
                format!("OSC-{}", self.count_modules_of_type(module_type) + 1)
            }
            ModuleType::Filter => format!("FILT-{}", self.count_modules_of_type(module_type) + 1),
            ModuleType::Reverb => format!("REV-{}", self.count_modules_of_type(module_type) + 1),
            ModuleType::Delay => format!("DLY-{}", self.count_modules_of_type(module_type) + 1),
            ModuleType::Output => format!("OUT-{}", self.count_modules_of_type(module_type) + 1),
            ModuleType::Sequencer => format!("SEQ-{}", self.count_modules_of_type(module_type) + 1),
        };

        let module = AudioModule::new(module_type, &name);
        self.modules.insert(module.id, module);
    }

    fn count_modules_of_type(&self, module_type: ModuleType) -> usize {
        self.modules
            .values()
            .filter(|m| m.module_type == module_type)
            .count()
    }
}
