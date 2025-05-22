use crate::modules::types::{AudioModule, ModuleType};

/// Reverb-specific functionality
impl AudioModule {
    /// Create a new reverb with default settings
    pub fn new_reverb(name: &str) -> Self {
        Self::new(ModuleType::Reverb, name)
    }

    /// Set room size (0.0 to 1.0)
    pub fn set_room_size(&mut self, room_size: f32) {
        if let Some(param) = self.get_parameter_mut("Room Size") {
            param.value = room_size.clamp(0.0, 1.0);
        }
    }

    /// Get current room size
    pub fn get_room_size(&self) -> f32 {
        self.get_parameter_value("Room Size")
    }

    /// Set damping (0.0 to 1.0)
    pub fn set_damping(&mut self, damping: f32) {
        if let Some(param) = self.get_parameter_mut("Damping") {
            param.value = damping.clamp(0.0, 1.0);
        }
    }

    /// Get current damping
    pub fn get_damping(&self) -> f32 {
        self.get_parameter_value("Damping")
    }

    /// Set wet level (0.0 to 1.0)
    pub fn set_wet_level(&mut self, wet: f32) {
        if let Some(param) = self.get_parameter_mut("Wet") {
            param.value = wet.clamp(0.0, 1.0);
        }
    }

    /// Get current wet level
    pub fn get_wet_level(&self) -> f32 {
        self.get_parameter_value("Wet")
    }
}

/// Delay-specific functionality
impl AudioModule {
    /// Create a new delay with default settings
    pub fn new_delay(name: &str) -> Self {
        Self::new(ModuleType::Delay, name)
    }

    /// Set delay time in seconds
    pub fn set_delay_time(&mut self, time: f32) {
        if let Some(param) = self.get_parameter_mut("Time") {
            param.value = time.clamp(param.min, param.max);
        }
    }

    /// Get current delay time
    pub fn get_delay_time(&self) -> f32 {
        self.get_parameter_value("Time")
    }

    /// Set feedback amount (0.0 to 0.95)
    pub fn set_feedback(&mut self, feedback: f32) {
        if let Some(param) = self.get_parameter_mut("Feedback") {
            param.value = feedback.clamp(0.0, 0.95);
        }
    }

    /// Get current feedback
    pub fn get_feedback(&self) -> f32 {
        self.get_parameter_value("Feedback")
    }

    /// Set delay time to match musical timing
    pub fn set_musical_timing(&mut self, bpm: f32, note_value: MusicalTiming) {
        let beat_duration = 60.0 / bpm;
        let delay_time = beat_duration * note_value.multiplier();
        self.set_delay_time(delay_time);
    }

    /// Set delay time in milliseconds (convenience method)
    pub fn set_delay_ms(&mut self, ms: f32) {
        self.set_delay_time(ms / 1000.0);
    }

    /// Get delay time in milliseconds
    pub fn get_delay_ms(&self) -> f32 {
        self.get_delay_time() * 1000.0
    }
}

/// Musical timing values for delay sync
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MusicalTiming {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    DottedHalf,
    DottedQuarter,
    DottedEighth,
    TripletQuarter,
    TripletEighth,
    TripletSixteenth,
}

impl MusicalTiming {
    pub fn multiplier(&self) -> f32 {
        match self {
            Self::Whole => 4.0,
            Self::Half => 2.0,
            Self::Quarter => 1.0,
            Self::Eighth => 0.5,
            Self::Sixteenth => 0.25,
            Self::ThirtySecond => 0.125,
            Self::DottedHalf => 3.0,
            Self::DottedQuarter => 1.5,
            Self::DottedEighth => 0.75,
            Self::TripletQuarter => 2.0 / 3.0,
            Self::TripletEighth => 1.0 / 3.0,
            Self::TripletSixteenth => 1.0 / 6.0,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Whole => "1/1",
            Self::Half => "1/2",
            Self::Quarter => "1/4",
            Self::Eighth => "1/8",
            Self::Sixteenth => "1/16",
            Self::ThirtySecond => "1/32",
            Self::DottedHalf => "1/2.",
            Self::DottedQuarter => "1/4.",
            Self::DottedEighth => "1/8.",
            Self::TripletQuarter => "1/4T",
            Self::TripletEighth => "1/8T",
            Self::TripletSixteenth => "1/16T",
        }
    }
}

/// Reverb preset factory
pub struct ReverbPresets;

impl ReverbPresets {
    /// Small room reverb
    pub fn small_room() -> AudioModule {
        let mut reverb = AudioModule::new_reverb("Small Room");
        reverb.set_room_size(0.3);
        reverb.set_damping(0.6);
        reverb.set_wet_level(0.3);
        reverb
    }

    /// Medium hall reverb
    pub fn medium_hall() -> AudioModule {
        let mut reverb = AudioModule::new_reverb("Medium Hall");
        reverb.set_room_size(0.6);
        reverb.set_damping(0.4);
        reverb.set_wet_level(0.4);
        reverb
    }

    /// Large cathedral reverb
    pub fn cathedral() -> AudioModule {
        let mut reverb = AudioModule::new_reverb("Cathedral");
        reverb.set_room_size(0.9);
        reverb.set_damping(0.2);
        reverb.set_wet_level(0.5);
        reverb
    }

    /// Plate reverb emulation
    pub fn plate() -> AudioModule {
        let mut reverb = AudioModule::new_reverb("Plate");
        reverb.set_room_size(0.4);
        reverb.set_damping(0.8);
        reverb.set_wet_level(0.3);
        reverb
    }

    /// Spring reverb emulation
    pub fn spring() -> AudioModule {
        let mut reverb = AudioModule::new_reverb("Spring");
        reverb.set_room_size(0.2);
        reverb.set_damping(0.5);
        reverb.set_wet_level(0.4);
        reverb
    }

    /// Ambient space reverb
    pub fn ambient() -> AudioModule {
        let mut reverb = AudioModule::new_reverb("Ambient");
        reverb.set_room_size(0.8);
        reverb.set_damping(0.3);
        reverb.set_wet_level(0.6);
        reverb
    }
}

/// Delay preset factory
pub struct DelayPresets;

impl DelayPresets {
    /// Short slap delay
    pub fn slap_delay() -> AudioModule {
        let mut delay = AudioModule::new_delay("Slap");
        delay.set_delay_ms(80.0);
        delay.set_feedback(0.2);
        delay.set_wet_level(0.3);
        delay
    }

    /// Echo delay
    pub fn echo() -> AudioModule {
        let mut delay = AudioModule::new_delay("Echo");
        delay.set_delay_ms(250.0);
        delay.set_feedback(0.4);
        delay.set_wet_level(0.4);
        delay
    }

    /// Long ambient delay
    pub fn ambient_delay() -> AudioModule {
        let mut delay = AudioModule::new_delay("Ambient");
        delay.set_delay_time(0.8);
        delay.set_feedback(0.6);
        delay.set_wet_level(0.5);
        delay
    }

    /// Ping-pong delay (stereo)
    pub fn ping_pong() -> AudioModule {
        let mut delay = AudioModule::new_delay("Ping Pong");
        delay.set_delay_ms(150.0);
        delay.set_feedback(0.5);
        delay.set_wet_level(0.4);
        delay
    }

    /// Tempo-synced quarter note delay
    pub fn quarter_note(bpm: f32) -> AudioModule {
        let mut delay = AudioModule::new_delay("1/4 Note");
        delay.set_musical_timing(bpm, MusicalTiming::Quarter);
        delay.set_feedback(0.3);
        delay.set_wet_level(0.3);
        delay
    }

    /// Tempo-synced eighth note delay
    pub fn eighth_note(bpm: f32) -> AudioModule {
        let mut delay = AudioModule::new_delay("1/8 Note");
        delay.set_musical_timing(bpm, MusicalTiming::Eighth);
        delay.set_feedback(0.4);
        delay.set_wet_level(0.3);
        delay
    }

    /// Dotted eighth delay (classic U2 style)
    pub fn dotted_eighth(bpm: f32) -> AudioModule {
        let mut delay = AudioModule::new_delay("1/8. Note");
        delay.set_musical_timing(bpm, MusicalTiming::DottedEighth);
        delay.set_feedback(0.45);
        delay.set_wet_level(0.35);
        delay
    }
}

/// Effect chain utilities
pub struct EffectChain {
    pub effects: Vec<AudioModule>,
}

impl EffectChain {
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
        }
    }

    pub fn add_effect(&mut self, effect: AudioModule) {
        self.effects.push(effect);
    }

    pub fn remove_effect(&mut self, index: usize) -> Option<AudioModule> {
        if index < self.effects.len() {
            Some(self.effects.remove(index))
        } else {
            None
        }
    }

    pub fn move_effect(&mut self, from: usize, to: usize) {
        if from < self.effects.len() && to < self.effects.len() {
            let effect = self.effects.remove(from);
            self.effects.insert(to, effect);
        }
    }

    pub fn clear(&mut self) {
        self.effects.clear();
    }

    pub fn get_effect_mut(&mut self, index: usize) -> Option<&mut AudioModule> {
        self.effects.get_mut(index)
    }

    pub fn len(&self) -> usize {
        self.effects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}

impl Default for EffectChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Common effect combinations
pub struct EffectCombinations;

impl EffectCombinations {
    /// Classic guitar setup: delay -> reverb
    pub fn guitar_classic(bpm: f32) -> EffectChain {
        let mut chain = EffectChain::new();
        chain.add_effect(DelayPresets::dotted_eighth(bpm));
        chain.add_effect(ReverbPresets::medium_hall());
        chain
    }

    /// Ambient pad setup: long delay -> cathedral reverb
    pub fn ambient_pad() -> EffectChain {
        let mut chain = EffectChain::new();
        chain.add_effect(DelayPresets::ambient_delay());
        chain.add_effect(ReverbPresets::cathedral());
        chain
    }

    /// Vocal setup: slap delay -> plate reverb
    pub fn vocal_treatment() -> EffectChain {
        let mut chain = EffectChain::new();
        chain.add_effect(DelayPresets::slap_delay());
        chain.add_effect(ReverbPresets::plate());
        chain
    }

    /// Dub delay setup: long feedback delay
    pub fn dub_delay(bpm: f32) -> EffectChain {
        let mut chain = EffectChain::new();
        let mut delay = DelayPresets::quarter_note(bpm);
        delay.set_feedback(0.7);
        delay.set_wet_level(0.6);
        chain.add_effect(delay);
        chain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverb_creation() {
        let reverb = AudioModule::new_reverb("Test Reverb");
        assert_eq!(reverb.module_type, ModuleType::Reverb);
    }

    #[test]
    fn test_delay_creation() {
        let delay = AudioModule::new_delay("Test Delay");
        assert_eq!(delay.module_type, ModuleType::Delay);
    }

    #[test]
    fn test_musical_timing() {
        assert_eq!(MusicalTiming::Quarter.multiplier(), 1.0);
        assert_eq!(MusicalTiming::Eighth.multiplier(), 0.5);
        assert_eq!(MusicalTiming::DottedQuarter.multiplier(), 1.5);
    }

    #[test]
    fn test_delay_musical_timing() {
        let mut delay = AudioModule::new_delay("Test");
        delay.set_musical_timing(120.0, MusicalTiming::Quarter);
        assert_eq!(delay.get_delay_time(), 0.5); // 60/120 = 0.5 seconds per beat
    }

    #[test]
    fn test_effect_chain() {
        let mut chain = EffectChain::new();
        chain.add_effect(DelayPresets::echo());
        chain.add_effect(ReverbPresets::small_room());

        assert_eq!(chain.len(), 2);
        assert!(!chain.is_empty());

        chain.move_effect(0, 1);
        // Should still have 2 effects but in different order
        assert_eq!(chain.len(), 2);
    }
}
