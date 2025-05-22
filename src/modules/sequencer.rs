use crate::modules::types::{AudioModule, ModuleType};
use crate::utils::math::midi_to_freq;

/// Sequencer-specific functionality
impl AudioModule {
    /// Create a new sequencer with default settings
    pub fn new_sequencer(name: &str) -> Self {
        Self::new(ModuleType::Sequencer, name)
    }

    /// Set BPM (beats per minute)
    pub fn set_bpm(&mut self, bpm: f32) {
        if let Some(param) = self.get_parameter_mut("BPM") {
            param.value = bpm.clamp(param.min, param.max);
        }
    }

    /// Get current BPM
    pub fn get_bpm(&self) -> f32 {
        self.get_parameter_value("BPM")
    }

    /// Set number of steps
    pub fn set_steps(&mut self, steps: f32) {
        if let Some(param) = self.get_parameter_mut("Steps") {
            param.value = steps.clamp(param.min, param.max);
        }
    }

    /// Get number of steps
    pub fn get_steps(&self) -> usize {
        self.get_parameter_value("Steps") as usize
    }

    /// Set gate length (0.0 to 1.0, where 1.0 = full step length)
    pub fn set_gate(&mut self, gate: f32) {
        if let Some(param) = self.get_parameter_mut("Gate") {
            param.value = gate.clamp(0.1, 1.0);
        }
    }

    /// Get gate length
    pub fn get_gate(&self) -> f32 {
        self.get_parameter_value("Gate")
    }
}

/// Individual step in a sequence
#[derive(Debug, Clone)]
pub struct SequenceStep {
    pub active: bool,
    pub note: Option<f32>,      // MIDI note number
    pub velocity: f32,          // 0.0 to 1.0
    pub probability: f32,       // 0.0 to 1.0 (for probabilistic sequencing)
    pub micro_timing: f32,      // -0.5 to 0.5 (timing offset within step)
}

impl Default for SequenceStep {
    fn default() -> Self {
        Self {
            active: false,
            note: Some(60.0), // Middle C
            velocity: 0.8,
            probability: 1.0,
            micro_timing: 0.0,
        }
    }
}

impl SequenceStep {
    pub fn new(active: bool) -> Self {
        Self {
            active,
            ..Default::default()
        }
    }

    pub fn with_note(mut self, note: f32) -> Self {
        self.note = Some(note);
        self
    }

    pub fn with_velocity(mut self, velocity: f32) -> Self {
        self.velocity = velocity.clamp(0.0, 1.0);
        self
    }

    pub fn with_probability(mut self, probability: f32) -> Self {
        self.probability = probability.clamp(0.0, 1.0);
        self
    }

    pub fn toggle(&mut self) {
        self.active = !self.active;
    }

    pub fn should_trigger(&self) -> bool {
        self.active && (self.probability >= 1.0 || rand::random::<f32>() < self.probability)
    }
}

/// Step sequencer implementation
#[derive(Debug, Clone)]
pub struct StepSequencer {
    pub steps: Vec<SequenceStep>,
    pub current_step: usize,
    pub is_playing: bool,
    pub loop_enabled: bool,
    pub swing: f32,             // 0.0 to 1.0 (amount of swing/shuffle)
    pub step_resolution: StepResolution,
    
    // Internal timing
    samples_per_step: f64,
    sample_counter: f64,
    sample_rate: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StepResolution {
    Sixteenth,  // 16th notes
    Eighth,     // 8th notes
    Quarter,    // Quarter notes
    Triplet,    // 8th note triplets
}

impl StepResolution {
    pub fn steps_per_beat(&self) -> f32 {
        match self {
            Self::Sixteenth => 4.0,
            Self::Eighth => 2.0,
            Self::Quarter => 1.0,
            Self::Triplet => 3.0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Sixteenth => "1/16",
            Self::Eighth => "1/8",
            Self::Quarter => "1/4",
            Self::Triplet => "1/8T",
        }
    }
}

impl StepSequencer {
    pub fn new(num_steps: usize, sample_rate: f64) -> Self {
        Self {
            steps: vec![SequenceStep::default(); num_steps],
            current_step: 0,
            is_playing: false,
            loop_enabled: true,
            swing: 0.0,
            step_resolution: StepResolution::Sixteenth,
            samples_per_step: 0.0,
            sample_counter: 0.0,
            sample_rate,
        }
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        let beats_per_second = bpm as f64 / 60.0;
        let steps_per_second = beats_per_second * self.step_resolution.steps_per_beat() as f64;
        self.samples_per_step = self.sample_rate / steps_per_second;
    }

    pub fn play(&mut self) {
        self.is_playing = true;
    }

    pub fn stop(&mut self) {
        self.is_playing = false;
        self.current_step = 0;
        self.sample_counter = 0.0;
    }

    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    pub fn reset(&mut self) {
        self.current_step = 0;
        self.sample_counter = 0.0;
    }

    pub fn tick(&mut self) -> Option<SequenceEvent> {
        if !self.is_playing {
            return None;
        }

        self.sample_counter += 1.0;
        
        let mut step_length = self.samples_per_step;
        
        // Apply swing to odd steps
        if self.swing > 0.0 && self.current_step % 2 == 1 {
            step_length *= 1.0 + (self.swing as f64 * 0.5);
        }

        if self.sample_counter >= step_length {
            self.sample_counter -= step_length;
            
            let event = if let Some(step) = self.steps.get(self.current_step) {
                if step.should_trigger() {
                    Some(SequenceEvent {
                        step_index: self.current_step,
                        note: step.note,
                        velocity: step.velocity,
                        gate_length: step_length,
                    })
                } else {
                    None
                }
            } else {
                None
            };

            // Advance to next step
            self.current_step += 1;
            if self.current_step >= self.steps.len() {
                if self.loop_enabled {
                    self.current_step = 0;
                } else {
                    self.is_playing = false;
                }
            }

            event
        } else {
            None
        }
    }

    pub fn set_step(&mut self, index: usize, step: SequenceStep) {
        if index < self.steps.len() {
            self.steps[index] = step;
        }
    }

    pub fn toggle_step(&mut self, index: usize) {
        if let Some(step) = self.steps.get_mut(index) {
            step.toggle();
        }
    }

    pub fn clear_all_steps(&mut self) {
        for step in &mut self.steps {
            step.active = false;
        }
    }

    pub fn randomize(&mut self, density: f32) {
        for step in &mut self.steps {
            step.active = rand::random::<f32>() < density;
            if step.active {
                step.velocity = 0.5 + rand::random::<f32>() * 0.5; // Random velocity 0.5-1.0
            }
        }
    }

    pub fn set_pattern(&mut self, pattern: &[bool]) {
        for (i, &active) in pattern.iter().enumerate() {
            if let Some(step) = self.steps.get_mut(i) {
                step.active = active;
            }
        }
    }

    pub fn get_pattern(&self) -> Vec<bool> {
        self.steps.iter().map(|step| step.active).collect()
    }

    pub fn resize(&mut self, new_size: usize) {
        self.steps.resize(new_size, SequenceStep::default());
        if self.current_step >= new_size && new_size > 0 {
            self.current_step = 0;
        }
    }
}

/// Event generated by the sequencer
#[derive(Debug, Clone)]
pub struct SequenceEvent {
    pub step_index: usize,
    pub note: Option<f32>,
    pub velocity: f32,
    pub gate_length: f64,
}

impl SequenceEvent {
    pub fn frequency(&self) -> Option<f64> {
        self.note.map(|note| midi_to_freq(note as f64))
    }
}

/// Drum machine patterns
pub struct DrumPatterns;

impl DrumPatterns {
    /// Classic 4/4 kick pattern
    pub fn kick_4_4() -> Vec<bool> {
        vec![true, false, false, false, true, false, false, false,
             true, false, false, false, true, false, false, false]
    }

    /// Basic snare on 2 and 4
    pub fn snare_2_4() -> Vec<bool> {
        vec![false, false, false, false, true, false, false, false,
             false, false, false, false, true, false, false, false]
    }

    /// Hi-hat pattern
    pub fn hihat_16th() -> Vec<bool> {
        vec![true, true, true, true, true, true, true, true,
             true, true, true, true, true, true, true, true]
    }

    /// Hi-hat with accents
    pub fn hihat_accent() -> Vec<bool> {
        vec![true, false, true, false, true, false, true, false,
             true, false, true, false, true, false, true, false]
    }

    /// Breakbeat pattern
    pub fn breakbeat() -> Vec<bool> {
        vec![true, false, false, false, true, false, true, false,
             false, false, true, false, true, false, false, false]
    }
}

/// Melody patterns
pub struct MelodyPatterns;

impl MelodyPatterns {
    /// Simple C major arpeggio
    pub fn c_major_arp() -> Vec<Option<f32>> {
        vec![Some(60.0), Some(64.0), Some(67.0), Some(72.0),  // C E G C
             Some(67.0), Some(64.0), Some(60.0), None,
             Some(60.0), Some(64.0), Some(67.0), Some(72.0),
             Some(67.0), Some(64.0), Some(60.0), None]
    }

    /// Pentatonic scale run
    pub fn pentatonic_run() -> Vec<Option<f32>> {
        vec![Some(60.0), Some(62.0), Some(65.0), Some(67.0),  // C D F G
             Some(69.0), Some(72.0), Some(69.0), Some(67.0),  // A C A G
             Some(65.0), Some(62.0), Some(60.0), None,
             None, None, None, None]
    }

    /// Bass line pattern
    pub fn bass_line() -> Vec<Option<f32>> {
        vec![Some(36.0), None, Some(36.0), None,  // C
             Some(41.0), None, Some(36.0), None,  // F C
             Some(38.0), None, Some(38.0), None,  // D
             Some(43.0), None, Some(38.0), None]  // G D
    }
}

/// Sequencer preset factory
pub struct SequencerPresets;

impl SequencerPresets {
    /// Basic 16-step drum sequencer
    pub fn drum_machine() -> StepSequencer {
        let mut seq = StepSequencer::new(16, 44100.0);
        seq.set_bpm(120.0);
        seq.step_resolution = StepResolution::Sixteenth;
        seq.set_pattern(&DrumPatterns::kick_4_4());
        seq
    }

    /// Melodic sequencer
    pub fn melody_seq() -> StepSequencer {
        let mut seq = StepSequencer::new(16, 44100.0);
        seq.set_bpm(120.0);
        seq.step_resolution = StepResolution::Eighth;
        
        let notes = MelodyPatterns::c_major_arp();
        for (i, note) in notes.iter().enumerate() {
            if let Some(step) = seq.steps.get_mut(i) {
                step.active = note.is_some();
                step.note = *note;
            }
        }
        
        seq
    }

    /// Bass sequencer
    pub fn bass_seq() -> StepSequencer {
        let mut seq = StepSequencer::new(16, 44100.0);
        seq.set_bpm(120.0);
        seq.step_resolution = StepResolution::Sixteenth;
        
        let notes = MelodyPatterns::bass_line();
        for (i, note) in notes.iter().enumerate() {
            if let Some(step) = seq.steps.get_mut(i) {
                step.active = note.is_some();
                step.note = *note;
                step.velocity = 0.9; // Bass hits hard
            }
        }
        
        seq
    }

    /// Ambient arpeggiator
    pub fn ambient_arp() -> StepSequencer {
        let mut seq = StepSequencer::new(8, 44100.0);
        seq.set_bpm(80.0);
        seq.step_resolution = StepResolution::Eighth;
        
        // Slow, ethereal arpeggio
        let notes = vec![Some(60.0), Some(67.0), Some(72.0), Some(76.0),
                        Some(79.0), Some(76.0), Some(72.0), Some(67.0)];
        
        for (i
