use crate::modules::types::{AudioModule, ModuleParameter, ModuleType};
use crate::utils::math::{freq_to_midi, midi_to_freq};

/// Oscillator-specific functionality
impl AudioModule {
    /// Create a new oscillator with default settings
    pub fn new_oscillator(name: &str) -> Self {
        Self::new(ModuleType::Oscillator, name)
    }

    /// Create an oscillator with specific waveform
    pub fn new_oscillator_with_waveform(name: &str, waveform: OscillatorWaveform) -> Self {
        let mut osc = Self::new_oscillator(name);
        if let Some(waveform_param) = osc.get_parameter_mut("Waveform") {
            waveform_param.value = waveform as i32 as f32;
        }
        osc
    }

    /// Set oscillator frequency in Hz
    pub fn set_frequency(&mut self, frequency: f32) {
        if let Some(freq_param) = self.get_parameter_mut("Frequency") {
            freq_param.value = frequency.clamp(freq_param.min, freq_param.max);
        }
    }

    /// Set oscillator frequency by MIDI note
    pub fn set_midi_note(&mut self, midi_note: f32) {
        let frequency = midi_to_freq(midi_note as f64) as f32;
        self.set_frequency(frequency);
    }

    /// Get current frequency
    pub fn get_frequency(&self) -> f32 {
        self.get_parameter_value("Frequency")
    }

    /// Get current frequency as MIDI note
    pub fn get_midi_note(&self) -> f32 {
        freq_to_midi(self.get_frequency() as f64) as f32
    }

    /// Set waveform type
    pub fn set_waveform(&mut self, waveform: OscillatorWaveform) {
        if let Some(waveform_param) = self.get_parameter_mut("Waveform") {
            waveform_param.value = waveform as i32 as f32;
        }
    }

    /// Get current waveform
    pub fn get_waveform(&self) -> OscillatorWaveform {
        let waveform_val = self.get_parameter_value("Waveform") as i32;
        match waveform_val {
            0 => OscillatorWaveform::Sine,
            1 => OscillatorWaveform::Sawtooth,
            2 => OscillatorWaveform::Square,
            3 => OscillatorWaveform::Triangle,
            _ => OscillatorWaveform::Sine,
        }
    }

    /// Set amplitude (0.0 to 1.0)
    pub fn set_amplitude(&mut self, amplitude: f32) {
        if let Some(amp_param) = self.get_parameter_mut("Amplitude") {
            amp_param.value = amplitude.clamp(0.0, 1.0);
        }
    }

    /// Get current amplitude
    pub fn get_amplitude(&self) -> f32 {
        self.get_parameter_value("Amplitude")
    }

    /// Detune the oscillator by cents (-100 to +100)
    pub fn detune_cents(&mut self, cents: f32) {
        let current_freq = self.get_frequency();
        let semitone_ratio = 2.0_f32.powf(1.0 / 12.0);
        let cent_ratio = semitone_ratio.powf(cents / 100.0);
        self.set_frequency(current_freq * cent_ratio);
    }

    /// Set oscillator to a musical note name (like "C4", "A#3")
    pub fn set_note(&mut self, note: &str) -> Result<(), String> {
        match parse_note_name(note) {
            Some(midi_note) => {
                self.set_midi_note(midi_note);
                Ok(())
            }
            None => Err(format!("Invalid note name: {}", note)),
        }
    }
}

/// Oscillator waveform types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OscillatorWaveform {
    Sine = 0,
    Sawtooth = 1,
    Square = 2,
    Triangle = 3,
}

impl OscillatorWaveform {
    pub fn all() -> Vec<Self> {
        vec![Self::Sine, Self::Sawtooth, Self::Square, Self::Triangle]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Sine => "Sine",
            Self::Sawtooth => "Sawtooth",
            Self::Square => "Square",
            Self::Triangle => "Triangle",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Sine => "~",
            Self::Sawtooth => "/|",
            Self::Square => "⊔⊓",
            Self::Triangle => "/\\",
        }
    }
}

/// Parse note names like "C4", "A#3", "Bb2" into MIDI note numbers
fn parse_note_name(note: &str) -> Option<f32> {
    if note.len() < 2 {
        return None;
    }

    let note_chars: Vec<char> = note.chars().collect();
    let note_letter = note_chars[0].to_ascii_uppercase();

    // Get the base note value (C=0, D=2, E=4, F=5, G=7, A=9, B=11)
    let base_note = match note_letter {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };

    let mut offset = 0;
    let mut octave_start_idx = 1;

    // Check for accidentals
    if note_chars.len() > 2 {
        match note_chars[1] {
            '#' => {
                offset = 1;
                octave_start_idx = 2;
            }
            'b' => {
                offset = -1;
                octave_start_idx = 2;
            }
            _ => {}
        }
    }

    // Parse octave number
    let octave_str: String = note_chars[octave_start_idx..].iter().collect();
    let octave: i32 = octave_str.parse().ok()?;

    // Calculate MIDI note number
    // MIDI note 60 = C4, so C0 = 12
    let midi_note = (octave + 1) * 12 + base_note + offset;

    if midi_note >= 0 && midi_note <= 127 {
        Some(midi_note as f32)
    } else {
        None
    }
}

/// Convert MIDI note number to note name
pub fn midi_to_note_name(midi_note: f32) -> String {
    let note_names = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let midi_int = midi_note.round() as i32;
    let octave = (midi_int / 12) - 1;
    let note_index = (midi_int % 12) as usize;

    format!("{}{}", note_names[note_index], octave)
}

/// Oscillator preset factory
pub struct OscillatorPresets;

impl OscillatorPresets {
    /// Create a basic sine wave oscillator
    pub fn sine_lead() -> AudioModule {
        let mut osc = AudioModule::new_oscillator("Sine Lead");
        osc.set_frequency(440.0);
        osc.set_amplitude(0.7);
        osc.set_waveform(OscillatorWaveform::Sine);
        osc
    }

    /// Create a sawtooth bass oscillator
    pub fn saw_bass() -> AudioModule {
        let mut osc = AudioModule::new_oscillator("Saw Bass");
        osc.set_frequency(110.0); // Low A
        osc.set_amplitude(0.8);
        osc.set_waveform(OscillatorWaveform::Sawtooth);
        osc
    }

    /// Create a square wave lead
    pub fn square_lead() -> AudioModule {
        let mut osc = AudioModule::new_oscillator("Square Lead");
        osc.set_frequency(880.0); // High A
        osc.set_amplitude(0.6);
        osc.set_waveform(OscillatorWaveform::Square);
        osc
    }

    /// Create a triangle wave pad
    pub fn triangle_pad() -> AudioModule {
        let mut osc = AudioModule::new_oscillator("Triangle Pad");
        osc.set_frequency(220.0);
        osc.set_amplitude(0.5);
        osc.set_waveform(OscillatorWaveform::Triangle);
        osc
    }

    /// Create an oscillator tuned to a specific musical note
    pub fn tuned_to_note(note: &str, waveform: OscillatorWaveform) -> Result<AudioModule, String> {
        let mut osc = AudioModule::new_oscillator(&format!("{} {}", waveform.name(), note));
        osc.set_note(note)?;
        osc.set_waveform(waveform);
        osc.set_amplitude(0.7);
        Ok(osc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_parsing() {
        assert_eq!(parse_note_name("C4"), Some(60.0));
        assert_eq!(parse_note_name("A4"), Some(69.0));
        assert_eq!(parse_note_name("C#4"), Some(61.0));
        assert_eq!(parse_note_name("Bb3"), Some(58.0));
        assert_eq!(parse_note_name("G0"), Some(19.0));
    }

    #[test]
    fn test_midi_to_note_name() {
        assert_eq!(midi_to_note_name(60.0), "C4");
        assert_eq!(midi_to_note_name(69.0), "A4");
        assert_eq!(midi_to_note_name(61.0), "C#4");
    }

    #[test]
    fn test_oscillator_frequency_setting() {
        let mut osc = AudioModule::new_oscillator("Test");
        osc.set_frequency(440.0);
        assert_eq!(osc.get_frequency(), 440.0);

        osc.set_midi_note(69.0); // A4
        assert!((osc.get_frequency() - 440.0).abs() < 0.1);
    }

    #[test]
    fn test_oscillator_waveform() {
        let mut osc = AudioModule::new_oscillator("Test");
        osc.set_waveform(OscillatorWaveform::Sawtooth);
        assert_eq!(osc.get_waveform(), OscillatorWaveform::Sawtooth);
    }
}
