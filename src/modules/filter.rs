use crate::modules::types::{AudioModule, ModuleType};

/// Filter-specific functionality
impl AudioModule {
    /// Create a new filter with default settings
    pub fn new_filter(name: &str) -> Self {
        Self::new(ModuleType::Filter, name)
    }

    /// Create a filter with specific type
    pub fn new_filter_with_type(name: &str, filter_type: FilterType) -> Self {
        let mut filter = Self::new_filter(name);
        filter.set_filter_type(filter_type);
        filter
    }

    /// Set filter cutoff frequency in Hz
    pub fn set_cutoff(&mut self, cutoff: f32) {
        if let Some(cutoff_param) = self.get_parameter_mut("Cutoff") {
            cutoff_param.value = cutoff.clamp(cutoff_param.min, cutoff_param.max);
        }
    }

    /// Get current cutoff frequency
    pub fn get_cutoff(&self) -> f32 {
        self.get_parameter_value("Cutoff")
    }

    /// Set filter resonance (0.0 to 1.0)
    pub fn set_resonance(&mut self, resonance: f32) {
        if let Some(res_param) = self.get_parameter_mut("Resonance") {
            res_param.value = resonance.clamp(0.0, 1.0);
        }
    }

    /// Get current resonance
    pub fn get_resonance(&self) -> f32 {
        self.get_parameter_value("Resonance")
    }

    /// Set filter type
    pub fn set_filter_type(&mut self, filter_type: FilterType) {
        if let Some(type_param) = self.get_parameter_mut("Type") {
            type_param.value = filter_type as i32 as f32;
        }
    }

    /// Get current filter type
    pub fn get_filter_type(&self) -> FilterType {
        let type_val = self.get_parameter_value("Type") as i32;
        match type_val {
            0 => FilterType::Lowpass,
            1 => FilterType::Highpass,
            2 => FilterType::Bandpass,
            3 => FilterType::Notch,
            4 => FilterType::Allpass,
            _ => FilterType::Lowpass,
        }
    }

    /// Set cutoff frequency relative to a base frequency (useful for tracking oscillators)
    pub fn set_cutoff_relative(&mut self, base_freq: f32, multiplier: f32) {
        let target_cutoff = base_freq * multiplier;
        self.set_cutoff(target_cutoff);
    }

    /// Sweep the cutoff frequency over time (for filter sweeps)
    pub fn sweep_cutoff(&mut self, start_freq: f32, end_freq: f32, position: f32) {
        let position = position.clamp(0.0, 1.0);
        let current_cutoff = start_freq + (end_freq - start_freq) * position;
        self.set_cutoff(current_cutoff);
    }
}

/// Filter types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterType {
    Lowpass = 0,
    Highpass = 1,
    Bandpass = 2,
    Notch = 3,
    Allpass = 4,
}

impl FilterType {
    pub fn all() -> Vec<Self> {
        vec![
            Self::Lowpass,
            Self::Highpass,
            Self::Bandpass,
            Self::Notch,
            Self::Allpass,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Lowpass => "Lowpass",
            Self::Highpass => "Highpass",
            Self::Bandpass => "Bandpass",
            Self::Notch => "Notch",
            Self::Allpass => "Allpass",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Lowpass => "\\__",
            Self::Highpass => "__/",
            Self::Bandpass => "_/\\_",
            Self::Notch => "\\/_/",
            Self::Allpass => "~~~",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Lowpass => "Cuts high frequencies",
            Self::Highpass => "Cuts low frequencies",
            Self::Bandpass => "Allows middle frequencies",
            Self::Notch => "Cuts middle frequencies",
            Self::Allpass => "Changes phase only",
        }
    }
}

/// Filter characteristic curve types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterCurve {
    Butter6dB,     // 6dB/octave (1-pole)
    Butter12dB,    // 12dB/octave (2-pole)
    Butter18dB,    // 18dB/octave (3-pole)
    Butter24dB,    // 24dB/octave (4-pole)
    Moog,          // Moog-style filter
    StateVariable, // State variable filter
}

impl FilterCurve {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Butter6dB => "6dB/oct",
            Self::Butter12dB => "12dB/oct",
            Self::Butter18dB => "18dB/oct",
            Self::Butter24dB => "24dB/oct",
            Self::Moog => "Moog",
            Self::StateVariable => "State Var",
        }
    }

    pub fn rolloff_db_per_octave(&self) -> f32 {
        match self {
            Self::Butter6dB => 6.0,
            Self::Butter12dB => 12.0,
            Self::Butter18dB => 18.0,
            Self::Butter24dB => 24.0,
            Self::Moog => 24.0,
            Self::StateVariable => 12.0,
        }
    }
}

/// Filter preset factory
pub struct FilterPresets;

impl FilterPresets {
    /// Classic lowpass filter for bass
    pub fn bass_lowpass() -> AudioModule {
        let mut filter = AudioModule::new_filter("Bass LP");
        filter.set_cutoff(500.0);
        filter.set_resonance(0.3);
        filter.set_filter_type(FilterType::Lowpass);
        filter
    }

    /// Bright highpass filter for removing muddy low end
    pub fn high_shelf() -> AudioModule {
        let mut filter = AudioModule::new_filter("High Shelf");
        filter.set_cutoff(200.0);
        filter.set_resonance(0.1);
        filter.set_filter_type(FilterType::Highpass);
        filter
    }

    /// Resonant lowpass for classic synth sounds
    pub fn synth_lowpass() -> AudioModule {
        let mut filter = AudioModule::new_filter("Synth LP");
        filter.set_cutoff(1000.0);
        filter.set_resonance(0.7);
        filter.set_filter_type(FilterType::Lowpass);
        filter
    }

    /// Bandpass filter for vocal-like sounds
    pub fn vocal_bandpass() -> AudioModule {
        let mut filter = AudioModule::new_filter("Vocal BP");
        filter.set_cutoff(800.0);
        filter.set_resonance(0.5);
        filter.set_filter_type(FilterType::Bandpass);
        filter
    }

    /// Notch filter for removing specific frequencies
    pub fn notch_60hz() -> AudioModule {
        let mut filter = AudioModule::new_filter("60Hz Notch");
        filter.set_cutoff(60.0);
        filter.set_resonance(0.8);
        filter.set_filter_type(FilterType::Notch);
        filter
    }

    /// Telephonic effect filter
    pub fn telephone() -> AudioModule {
        let mut filter = AudioModule::new_filter("Telephone");
        filter.set_cutoff(3000.0);
        filter.set_resonance(0.2);
        filter.set_filter_type(FilterType::Bandpass);
        filter
    }

    /// Radio filter effect
    pub fn radio() -> AudioModule {
        let mut filter = AudioModule::new_filter("Radio");
        filter.set_cutoff(4000.0);
        filter.set_resonance(0.4);
        filter.set_filter_type(FilterType::Bandpass);
        filter
    }
}

/// Filter utilities
pub struct FilterUtils;

impl FilterUtils {
    /// Calculate filter frequency response at a given frequency
    pub fn frequency_response(
        filter_type: FilterType,
        cutoff: f32,
        resonance: f32,
        freq: f32,
    ) -> f32 {
        let normalized_freq = freq / cutoff;

        match filter_type {
            FilterType::Lowpass => {
                let response = 1.0 / (1.0 + normalized_freq.powi(2)).sqrt();
                response * (1.0 + resonance * (1.0 - response))
            }
            FilterType::Highpass => {
                let response = normalized_freq / (1.0 + normalized_freq.powi(2)).sqrt();
                response * (1.0 + resonance * response)
            }
            FilterType::Bandpass => {
                let q = 1.0 / (1.0 - resonance * 0.99); // Convert resonance to Q
                let response = (normalized_freq / q)
                    / ((1.0 - normalized_freq.powi(2)).powi(2) + (normalized_freq / q).powi(2))
                        .sqrt();
                response
            }
            FilterType::Notch => {
                let q = 1.0 / (1.0 - resonance * 0.99);
                let response = (1.0 - normalized_freq.powi(2)).abs()
                    / ((1.0 - normalized_freq.powi(2)).powi(2) + (normalized_freq / q).powi(2))
                        .sqrt();
                response
            }
            FilterType::Allpass => 1.0, // Allpass doesn't change amplitude
        }
    }

    /// Calculate the -3dB frequency for a filter
    pub fn calculate_3db_frequency(filter_type: FilterType, cutoff: f32, resonance: f32) -> f32 {
        match filter_type {
            FilterType::Lowpass | FilterType::Highpass => {
                // For basic filters, -3dB point is approximately the cutoff
                cutoff * (1.0 + resonance * 0.1)
            }
            FilterType::Bandpass => {
                // Bandpass -3dB points depend on Q
                let q = 1.0 / (1.0 - resonance * 0.99);
                cutoff / q.sqrt()
            }
            FilterType::Notch => cutoff, // Notch doesn't have a traditional -3dB point
            FilterType::Allpass => cutoff, // Allpass doesn't change amplitude
        }
    }

    /// Convert cutoff frequency to normalized value (0.0 to 1.0)
    pub fn normalize_cutoff(cutoff: f32, sample_rate: f32) -> f32 {
        (cutoff / (sample_rate * 0.5)).clamp(0.0, 1.0)
    }

    /// Convert normalized cutoff back to frequency
    pub fn denormalize_cutoff(normalized: f32, sample_rate: f32) -> f32 {
        normalized * sample_rate * 0.5
    }

    /// Calculate filter coefficients for a simple biquad filter
    pub fn calculate_biquad_coefficients(
        filter_type: FilterType,
        cutoff: f32,
        resonance: f32,
        sample_rate: f32,
    ) -> BiquadCoefficients {
        use std::f32::consts::PI;

        let omega = 2.0 * PI * cutoff / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * (1.0 / (1.0 - resonance * 0.99)));

        let (b0, b1, b2, a0, a1, a2) = match filter_type {
            FilterType::Lowpass => (
                (1.0 - cos_omega) / 2.0,
                1.0 - cos_omega,
                (1.0 - cos_omega) / 2.0,
                1.0 + alpha,
                -2.0 * cos_omega,
                1.0 - alpha,
            ),
            FilterType::Highpass => (
                (1.0 + cos_omega) / 2.0,
                -(1.0 + cos_omega),
                (1.0 + cos_omega) / 2.0,
                1.0 + alpha,
                -2.0 * cos_omega,
                1.0 - alpha,
            ),
            FilterType::Bandpass => (
                sin_omega / 2.0,
                0.0,
                -sin_omega / 2.0,
                1.0 + alpha,
                -2.0 * cos_omega,
                1.0 - alpha,
            ),
            FilterType::Notch => (
                1.0,
                -2.0 * cos_omega,
                1.0,
                1.0 + alpha,
                -2.0 * cos_omega,
                1.0 - alpha,
            ),
            FilterType::Allpass => (
                1.0 - alpha,
                -2.0 * cos_omega,
                1.0 + alpha,
                1.0 + alpha,
                -2.0 * cos_omega,
                1.0 - alpha,
            ),
        };

        BiquadCoefficients {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }
}

/// Biquad filter coefficients
#[derive(Debug, Clone, Copy)]
pub struct BiquadCoefficients {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}

/// Filter envelope for automated sweeps
#[derive(Debug, Clone)]
pub struct FilterEnvelope {
    pub start_cutoff: f32,
    pub end_cutoff: f32,
    pub duration: f32,
    pub curve: FilterEnvelopeCurve,
    current_time: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum FilterEnvelopeCurve {
    Linear,
    Exponential,
    Logarithmic,
}

impl FilterEnvelope {
    pub fn new(start_cutoff: f32, end_cutoff: f32, duration: f32) -> Self {
        Self {
            start_cutoff,
            end_cutoff,
            duration,
            curve: FilterEnvelopeCurve::Linear,
            current_time: 0.0,
        }
    }

    pub fn with_curve(mut self, curve: FilterEnvelopeCurve) -> Self {
        self.curve = curve;
        self
    }

    pub fn tick(&mut self, delta_time: f32) -> f32 {
        self.current_time += delta_time;
        let progress = (self.current_time / self.duration).clamp(0.0, 1.0);

        let curved_progress = match self.curve {
            FilterEnvelopeCurve::Linear => progress,
            FilterEnvelopeCurve::Exponential => progress * progress,
            FilterEnvelopeCurve::Logarithmic => progress.sqrt(),
        };

        self.start_cutoff + (self.end_cutoff - self.start_cutoff) * curved_progress
    }

    pub fn reset(&mut self) {
        self.current_time = 0.0;
    }

    pub fn is_finished(&self) -> bool {
        self.current_time >= self.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_creation() {
        let filter = AudioModule::new_filter("Test Filter");
        assert_eq!(filter.module_type, ModuleType::Filter);
        assert_eq!(filter.name, "Test Filter");
    }

    #[test]
    fn test_filter_parameters() {
        let mut filter = AudioModule::new_filter("Test");
        filter.set_cutoff(1000.0);
        filter.set_resonance(0.5);
        filter.set_filter_type(FilterType::Lowpass);

        assert_eq!(filter.get_cutoff(), 1000.0);
        assert_eq!(filter.get_resonance(), 0.5);
        assert_eq!(filter.get_filter_type(), FilterType::Lowpass);
    }

    #[test]
    fn test_frequency_response() {
        let response = FilterUtils::frequency_response(FilterType::Lowpass, 1000.0, 0.0, 1000.0);
        assert!((response - 0.707).abs() < 0.01); // -3dB point
    }

    #[test]
    fn test_filter_envelope() {
        let mut envelope = FilterEnvelope::new(100.0, 1000.0, 1.0);

        assert_eq!(envelope.tick(0.0), 100.0);
        assert_eq!(envelope.tick(0.5), 550.0);
        assert_eq!(envelope.tick(0.5), 1000.0);
        assert!(envelope.is_finished());
    }
}
