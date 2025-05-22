use crate::modules::types::{AudioModule, ModuleType};
use crate::utils::math::{amp_to_db, db_to_amp};

/// Output-specific functionality
impl AudioModule {
    /// Create a new output module with default settings
    pub fn new_output(name: &str) -> Self {
        Self::new(ModuleType::Output, name)
    }

    /// Set output volume (0.0 to 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        if let Some(param) = self.get_parameter_mut("Volume") {
            param.value = volume.clamp(0.0, 1.0);
        }
    }

    /// Get current volume
    pub fn get_volume(&self) -> f32 {
        self.get_parameter_value("Volume")
    }

    /// Set volume in dB (-60dB to 0dB)
    pub fn set_volume_db(&mut self, db: f32) {
        let linear = if db <= -60.0 {
            0.0
        } else {
            db_to_amp(db.clamp(-60.0, 0.0))
        };
        self.set_volume(linear as f32);
    }

    /// Get volume in dB
    pub fn get_volume_db(&self) -> f32 {
        amp_to_db(self.get_volume() as f64) as f32
    }

    /// Set pan position (0.0 = left, 0.5 = center, 1.0 = right)
    pub fn set_pan(&mut self, pan: f32) {
        if let Some(param) = self.get_parameter_mut("Pan") {
            param.value = pan.clamp(0.0, 1.0);
        }
    }

    /// Get current pan position
    pub fn get_pan(&self) -> f32 {
        self.get_parameter_value("Pan")
    }

    /// Set pan in percentage (-100% to +100%)
    pub fn set_pan_percent(&mut self, percent: f32) {
        let normalized = (percent + 100.0) / 200.0;
        self.set_pan(normalized.clamp(0.0, 1.0));
    }

    /// Get pan in percentage
    pub fn get_pan_percent(&self) -> f32 {
        (self.get_pan() - 0.5) * 200.0
    }

    /// Mute the output
    pub fn mute(&mut self) {
        self.enabled = false;
    }

    /// Unmute the output
    pub fn unmute(&mut self) {
        self.enabled = true;
    }

    /// Toggle mute state
    pub fn toggle_mute(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Check if output is muted
    pub fn is_muted(&self) -> bool {
        !self.enabled
    }

    /// Calculate stereo gain values from pan and volume
    pub fn get_stereo_gains(&self) -> (f32, f32) {
        if !self.enabled {
            return (0.0, 0.0);
        }

        let volume = self.get_volume();
        let pan = self.get_pan();

        // Equal power panning
        let left_gain = volume * (1.0 - pan).sqrt();
        let right_gain = volume * pan.sqrt();

        (left_gain, right_gain)
    }

    /// Apply volume and pan to a mono signal, returning stereo
    pub fn process_mono_to_stereo(&self, input: f32) -> (f32, f32) {
        let (left_gain, right_gain) = self.get_stereo_gains();
        (input * left_gain, input * right_gain)
    }

    /// Apply volume to a stereo signal
    pub fn process_stereo(&self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (0.0, 0.0);
        }

        let volume = self.get_volume();
        (left * volume, right * volume)
    }
}

/// Output types for different routing scenarios
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputType {
    MainOutput,
    AuxSend,
    HeadphoneOut,
    MonitorOut,
    RecordOut,
}

impl OutputType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::MainOutput => "Main Out",
            Self::AuxSend => "Aux Send",
            Self::HeadphoneOut => "Headphones",
            Self::MonitorOut => "Monitor",
            Self::RecordOut => "Record",
        }
    }

    pub fn default_volume(&self) -> f32 {
        match self {
            Self::MainOutput => 0.8,
            Self::AuxSend => 0.0,
            Self::HeadphoneOut => 0.7,
            Self::MonitorOut => 0.6,
            Self::RecordOut => 0.8,
        }
    }
}

/// Output metering for level monitoring
#[derive(Debug, Clone)]
pub struct OutputMeter {
    pub peak_left: f32,
    pub peak_right: f32,
    pub rms_left: f32,
    pub rms_right: f32,
    peak_hold_left: f32,
    peak_hold_right: f32,
    peak_hold_time: f32,
    rms_buffer_left: Vec<f32>,
    rms_buffer_right: Vec<f32>,
    buffer_index: usize,
}

impl OutputMeter {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            peak_left: 0.0,
            peak_right: 0.0,
            rms_left: 0.0,
            rms_right: 0.0,
            peak_hold_left: 0.0,
            peak_hold_right: 0.0,
            peak_hold_time: 0.0,
            rms_buffer_left: vec![0.0; buffer_size],
            rms_buffer_right: vec![0.0; buffer_size],
            buffer_index: 0,
        }
    }

    pub fn process(&mut self, left: f32, right: f32, delta_time: f32) {
        // Update peak levels
        self.peak_left = left.abs().max(self.peak_left * 0.999); // Slow decay
        self.peak_right = right.abs().max(self.peak_right * 0.999);

        // Peak hold logic
        if left.abs() > self.peak_hold_left {
            self.peak_hold_left = left.abs();
            self.peak_hold_time = 2.0; // Hold for 2 seconds
        }
        if right.abs() > self.peak_hold_right {
            self.peak_hold_right = right.abs();
            self.peak_hold_time = 2.0;
        }

        // Decay peak hold
        self.peak_hold_time -= delta_time;
        if self.peak_hold_time <= 0.0 {
            self.peak_hold_left *= 0.95;
            self.peak_hold_right *= 0.95;
        }

        // Update RMS buffers
        self.rms_buffer_left[self.buffer_index] = left * left;
        self.rms_buffer_right[self.buffer_index] = right * right;
        self.buffer_index = (self.buffer_index + 1) % self.rms_buffer_left.len();

        // Calculate RMS
        let sum_left: f32 = self.rms_buffer_left.iter().sum();
        let sum_right: f32 = self.rms_buffer_right.iter().sum();
        self.rms_left = (sum_left / self.rms_buffer_left.len() as f32).sqrt();
        self.rms_right = (sum_right / self.rms_buffer_right.len() as f32).sqrt();
    }

    pub fn reset(&mut self) {
        self.peak_left = 0.0;
        self.peak_right = 0.0;
        self.rms_left = 0.0;
        self.rms_right = 0.0;
        self.peak_hold_left = 0.0;
        self.peak_hold_right = 0.0;
        self.peak_hold_time = 0.0;
        self.rms_buffer_left.fill(0.0);
        self.rms_buffer_right.fill(0.0);
        self.buffer_index = 0;
    }

    /// Check if signal is clipping (>= 1.0)
    pub fn is_clipping(&self) -> (bool, bool) {
        (self.peak_left >= 1.0, self.peak_right >= 1.0)
    }

    /// Get peak levels in dB
    pub fn get_peak_db(&self) -> (f32, f32) {
        (
            amp_to_db(self.peak_left as f64) as f32,
            amp_to_db(self.peak_right as f64) as f32,
        )
    }

    /// Get RMS levels in dB
    pub fn get_rms_db(&self) -> (f32, f32) {
        (
            amp_to_db(self.rms_left as f64) as f32,
            amp_to_db(self.rms_right as f64) as f32,
        )
    }
}

/// Output limiter to prevent clipping
#[derive(Debug, Clone)]
pub struct OutputLimiter {
    threshold: f32,
    ratio: f32,
    attack_time: f32,
    release_time: f32,
    envelope: f32,
    sample_rate: f32,
}

impl OutputLimiter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            threshold: 0.9,     // Start limiting at -1dB
            ratio: 10.0,        // 10:1 ratio (almost limiting)
            attack_time: 0.001, // 1ms attack
            release_time: 0.1,  // 100ms release
            envelope: 0.0,
            sample_rate,
        }
    }

    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        let max_level = left.abs().max(right.abs());

        if max_level > self.threshold {
            let over_threshold = max_level - self.threshold;
            let target_gain = 1.0 - (over_threshold * (self.ratio - 1.0) / self.ratio);

            // Envelope follower
            let attack_coeff = (-1.0 / (self.attack_time * self.sample_rate)).exp();
            let release_coeff = (-1.0 / (self.release_time * self.sample_rate)).exp();

            if target_gain < self.envelope {
                self.envelope = target_gain + (self.envelope - target_gain) * attack_coeff;
            } else {
                self.envelope = target_gain + (self.envelope - target_gain) * release_coeff;
            }
        } else {
            // Release
            let release_coeff = (-1.0 / (self.release_time * self.sample_rate)).exp();
            self.envelope = 1.0 + (self.envelope - 1.0) * release_coeff;
        }

        let gain = self.envelope.clamp(0.0, 1.0);
        (left * gain, right * gain)
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.1, 1.0);
    }

    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(1.0, 20.0);
    }
}

/// Output preset factory
pub struct OutputPresets;

impl OutputPresets {
    /// Standard main output
    pub fn main_out() -> AudioModule {
        let mut output = AudioModule::new_output("Main Out");
        output.set_volume(0.8);
        output.set_pan(0.5); // Center
        output
    }

    /// Headphone output with slightly lower volume
    pub fn headphone_out() -> AudioModule {
        let mut output = AudioModule::new_output("Headphones");
        output.set_volume(0.7);
        output.set_pan(0.5);
        output
    }

    /// Monitor output for studio monitors
    pub fn monitor_out() -> AudioModule {
        let mut output = AudioModule::new_output("Monitors");
        output.set_volume(0.6);
        output.set_pan(0.5);
        output
    }

    /// Aux send output (starts muted)
    pub fn aux_send(name: &str) -> AudioModule {
        let mut output = AudioModule::new_output(name);
        output.set_volume(0.0);
        output.set_pan(0.5);
        output
    }

    /// Record output
    pub fn record_out() -> AudioModule {
        let mut output = AudioModule::new_output("Record");
        output.set_volume(0.8);
        output.set_pan(0.5);
        output
    }

    /// Left-panned output
    pub fn left_out() -> AudioModule {
        let mut output = AudioModule::new_output("Left Out");
        output.set_volume(0.8);
        output.set_pan(0.0); // Full left
        output
    }

    /// Right-panned output
    pub fn right_out() -> AudioModule {
        let mut output = AudioModule::new_output("Right Out");
        output.set_volume(0.8);
        output.set_pan(1.0); // Full right
        output
    }
}

/// Output routing matrix for complex setups
#[derive(Debug, Clone)]
pub struct OutputMatrix {
    outputs: Vec<AudioModule>,
    routing: Vec<Vec<f32>>, // [input][output] = gain
}

impl OutputMatrix {
    pub fn new() -> Self {
        Self {
            outputs: Vec::new(),
            routing: Vec::new(),
        }
    }

    pub fn add_output(&mut self, output: AudioModule) {
        self.outputs.push(output);

        // Add column to routing matrix
        for row in &mut self.routing {
            row.push(0.0);
        }
    }

    pub fn add_input(&mut self) -> usize {
        let input_index = self.routing.len();
        self.routing.push(vec![0.0; self.outputs.len()]);
        input_index
    }

    pub fn set_routing(&mut self, input: usize, output: usize, gain: f32) {
        if input < self.routing.len() && output < self.routing[input].len() {
            self.routing[input][output] = gain.clamp(0.0, 1.0);
        }
    }

    pub fn get_routing(&self, input: usize, output: usize) -> f32 {
        self.routing
            .get(input)
            .and_then(|row| row.get(output))
            .copied()
            .unwrap_or(0.0)
    }

    pub fn process(&self, inputs: &[f32]) -> Vec<(f32, f32)> {
        let mut outputs = vec![(0.0, 0.0); self.outputs.len()];

        for (input_idx, &input_value) in inputs.iter().enumerate() {
            if let Some(routing_row) = self.routing.get(input_idx) {
                for (output_idx, &gain) in routing_row.iter().enumerate() {
                    if gain > 0.0 {
                        if let Some(output_module) = self.outputs.get(output_idx) {
                            let (left, right) =
                                output_module.process_mono_to_stereo(input_value * gain);
                            outputs[output_idx].0 += left;
                            outputs[output_idx].1 += right;
                        }
                    }
                }
            }
        }

        outputs
    }
}

impl Default for OutputMatrix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_creation() {
        let output = AudioModule::new_output("Test Output");
        assert_eq!(output.module_type, ModuleType::Output);
        assert_eq!(output.name, "Test Output");
    }

    #[test]
    fn test_volume_setting() {
        let mut output = AudioModule::new_output("Test");
        output.set_volume(0.5);
        assert_eq!(output.get_volume(), 0.5);

        output.set_volume_db(-6.0);
        let volume_linear = output.get_volume();
        assert!((volume_linear - 0.501).abs() < 0.01); // -6dB ≈ 0.501
    }

    #[test]
    fn test_pan_setting() {
        let mut output = AudioModule::new_output("Test");
        output.set_pan(0.75);
        assert_eq!(output.get_pan(), 0.75);

        output.set_pan_percent(50.0); // 50% right
        assert_eq!(output.get_pan(), 0.75);
    }

    #[test]
    fn test_stereo_processing() {
        let mut output = AudioModule::new_output("Test");
        output.set_volume(0.5);
        output.set_pan(0.5); // Center

        let (left, right) = output.process_mono_to_stereo(1.0);
        assert!((left - 0.3536).abs() < 0.01); // sqrt(0.5) * 0.5 ≈ 0.3536
        assert!((right - 0.3536).abs() < 0.01);
    }

    #[test]
    fn test_muting() {
        let mut output = AudioModule::new_output("Test");
        assert!(!output.is_muted());

        output.mute();
        assert!(output.is_muted());

        let (left, right) = output.process_mono_to_stereo(1.0);
        assert_eq!(left, 0.0);
        assert_eq!(right, 0.0);
    }

    #[test]
    fn test_output_meter() {
        let mut meter = OutputMeter::new(10);
        meter.process(0.5, -0.5, 0.01);

        assert!(meter.peak_left > 0.0);
        assert!(meter.peak_right > 0.0);
        assert!(meter.rms_left > 0.0);
        assert!(meter.rms_right > 0.0);
    }

    #[test]
    fn test_output_matrix() {
        let mut matrix = OutputMatrix::new();
        matrix.add_output(OutputPresets::main_out());
        matrix.add_output(OutputPresets::headphone_out());

        let input1 = matrix.add_input();
        let input2 = matrix.add_input();

        matrix.set_routing(input1, 0, 1.0); // Input 1 -> Main Out
        matrix.set_routing(input2, 1, 0.5); // Input 2 -> Headphones at half gain

        let outputs = matrix.process(&[0.8, 0.6]);
        assert_eq!(outputs.len(), 2);
    }
}
