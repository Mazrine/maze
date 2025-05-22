use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Represents a key binding action
#[derive(Debug, Clone, PartialEq)]
pub enum InputAction {
    // Navigation
    NextModule,
    PrevModule,
    NextParameter,
    PrevParameter,

    // Mode changes
    EnterParameterEdit,
    EnterModuleAdd,
    EnterConnection,
    ExitMode,

    // Parameter adjustment
    IncreaseParameterFine,
    DecreaseParameterFine,
    IncreaseParameterCoarse,
    DecreaseParameterCoarse,

    // Module management
    AddOscillator,
    AddFilter,
    AddReverb,
    AddDelay,
    AddOutput,
    AddSequencer,
    DeleteModule,

    // Project management
    SaveProject,
    LoadProject,
    NewProject,

    // Playback control
    PlayPause,
    Stop,
    Record,

    // System
    Quit,

    // Unknown action
    Unknown,
}

/// Convert key events to input actions
pub fn key_to_action(key: KeyEvent) -> InputAction {
    match (key.code, key.modifiers) {
        // Basic navigation
        (KeyCode::Tab, KeyModifiers::NONE) => InputAction::NextModule,
        (KeyCode::BackTab, _) => InputAction::PrevModule,
        (KeyCode::Up, KeyModifiers::NONE) => InputAction::PrevParameter,
        (KeyCode::Down, KeyModifiers::NONE) => InputAction::NextParameter,

        // Mode changes
        (KeyCode::Enter, KeyModifiers::NONE) => InputAction::EnterParameterEdit,
        (KeyCode::Esc, KeyModifiers::NONE) => InputAction::ExitMode,
        (KeyCode::Char('a'), KeyModifiers::NONE) => InputAction::EnterModuleAdd,
        (KeyCode::Char('c'), KeyModifiers::NONE) => InputAction::EnterConnection,

        // Parameter adjustment
        (KeyCode::Left, KeyModifiers::NONE) => InputAction::DecreaseParameterFine,
        (KeyCode::Right, KeyModifiers::NONE) => InputAction::IncreaseParameterFine,
        (KeyCode::Char('-'), KeyModifiers::NONE) => InputAction::DecreaseParameterCoarse,
        (KeyCode::Char('='), KeyModifiers::NONE) | (KeyCode::Char('+'), KeyModifiers::NONE) => {
            InputAction::IncreaseParameterCoarse
        }

        // Module creation (in add mode)
        (KeyCode::Char('o'), KeyModifiers::NONE) => InputAction::AddOscillator,
        (KeyCode::Char('f'), KeyModifiers::NONE) => InputAction::AddFilter,
        (KeyCode::Char('r'), KeyModifiers::NONE) => InputAction::AddReverb,
        (KeyCode::Char('d'), KeyModifiers::NONE) => InputAction::AddDelay,
        (KeyCode::Char('u'), KeyModifiers::NONE) => InputAction::AddOutput,
        (KeyCode::Char('s'), KeyModifiers::NONE) => InputAction::AddSequencer,

        // Module management
        (KeyCode::Delete, KeyModifiers::NONE) | (KeyCode::Char('x'), KeyModifiers::NONE) => {
            InputAction::DeleteModule
        }

        // Project management
        (KeyCode::Char('s'), KeyModifiers::CONTROL) => InputAction::SaveProject,
        (KeyCode::Char('o'), KeyModifiers::CONTROL) => InputAction::LoadProject,
        (KeyCode::Char('n'), KeyModifiers::CONTROL) => InputAction::NewProject,

        // Playback
        (KeyCode::Char(' '), KeyModifiers::NONE) => InputAction::PlayPause,
        (KeyCode::Char('.'), KeyModifiers::NONE) => InputAction::Stop,
        (KeyCode::Char('r'), KeyModifiers::CONTROL) => InputAction::Record,

        // System
        (KeyCode::Char('q'), KeyModifiers::NONE) => InputAction::Quit,
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => InputAction::Quit,

        _ => InputAction::Unknown,
    }
}

/// Check if a key combination is valid for the current mode
pub fn is_action_valid_for_mode(action: &InputAction, mode: &crate::app::modes::AppMode) -> bool {
    use crate::app::modes::AppMode;

    match mode {
        AppMode::ModuleView => matches!(
            action,
            InputAction::NextModule
                | InputAction::PrevModule
                | InputAction::EnterParameterEdit
                | InputAction::EnterModuleAdd
                | InputAction::EnterConnection
                | InputAction::DeleteModule
                | InputAction::SaveProject
                | InputAction::LoadProject
                | InputAction::NewProject
                | InputAction::PlayPause
                | InputAction::Stop
                | InputAction::Record
                | InputAction::Quit
        ),

        AppMode::ParameterEdit => matches!(
            action,
            InputAction::NextParameter
                | InputAction::PrevParameter
                | InputAction::IncreaseParameterFine
                | InputAction::DecreaseParameterFine
                | InputAction::IncreaseParameterCoarse
                | InputAction::DecreaseParameterCoarse
                | InputAction::ExitMode
                | InputAction::Quit
        ),

        AppMode::ModuleAdd => matches!(
            action,
            InputAction::AddOscillator
                | InputAction::AddFilter
                | InputAction::AddReverb
                | InputAction::AddDelay
                | InputAction::AddOutput
                | InputAction::AddSequencer
                | InputAction::ExitMode
                | InputAction::Quit
        ),

        AppMode::Connection => matches!(action, InputAction::ExitMode | InputAction::Quit),
    }
}

/// Get help text for available actions in a mode
pub fn get_mode_help(mode: &crate::app::modes::AppMode) -> Vec<(&'static str, &'static str)> {
    use crate::app::modes::AppMode;

    match mode {
        AppMode::ModuleView => vec![
            ("Tab", "Select module"),
            ("Enter", "Edit parameters"),
            ("A", "Add module"),
            ("C", "Connect modules"),
            ("Del/X", "Delete module"),
            ("Ctrl+S", "Save"),
            ("Ctrl+O", "Load"),
            ("Space", "Play/Pause"),
            ("Q", "Quit"),
        ],

        AppMode::ParameterEdit => vec![
            ("↑↓", "Select parameter"),
            ("←→", "Fine adjust"),
            ("-/+", "Coarse adjust"),
            ("Esc", "Back to modules"),
        ],

        AppMode::ModuleAdd => vec![
            ("O", "Oscillator"),
            ("F", "Filter"),
            ("R", "Reverb"),
            ("D", "Delay"),
            ("U", "Output"),
            ("S", "Sequencer"),
            ("Esc", "Cancel"),
        ],

        AppMode::Connection => vec![("Esc", "Cancel"), ("(Connection UI coming soon!)", "")],
    }
}

/// Utility for handling rapid key repeats
pub struct KeyRepeatHandler {
    last_key: Option<KeyCode>,
    repeat_count: u32,
    last_time: std::time::Instant,
}

impl Default for KeyRepeatHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyRepeatHandler {
    pub fn new() -> Self {
        Self {
            last_key: None,
            repeat_count: 0,
            last_time: std::time::Instant::now(),
        }
    }

    /// Process a key event and return the multiplier for parameter changes
    pub fn process_key(&mut self, key: KeyCode) -> f32 {
        let now = std::time::Instant::now();
        let time_since_last = now.duration_since(self.last_time);

        if Some(key) == self.last_key && time_since_last.as_millis() < 200 {
            // Same key pressed quickly - increase multiplier
            self.repeat_count += 1;
        } else {
            // Different key or too much time passed - reset
            self.repeat_count = 0;
        }

        self.last_key = Some(key);
        self.last_time = now;

        // Return multiplier based on repeat count
        match self.repeat_count {
            0..=2 => 1.0,
            3..=5 => 2.0,
            6..=10 => 5.0,
            _ => 10.0,
        }
    }
}

/// Musical keyboard mapping for playing notes
pub fn key_to_note(key: KeyCode) -> Option<f64> {
    // Map QWERTY keyboard to piano keys (like a typical piano roll)
    match key {
        // Bottom row - C major scale
        KeyCode::Char('z') => Some(midi_to_freq(60)), // C4
        KeyCode::Char('x') => Some(midi_to_freq(62)), // D4
        KeyCode::Char('c') => Some(midi_to_freq(64)), // E4
        KeyCode::Char('v') => Some(midi_to_freq(65)), // F4
        KeyCode::Char('b') => Some(midi_to_freq(67)), // G4
        KeyCode::Char('n') => Some(midi_to_freq(69)), // A4
        KeyCode::Char('m') => Some(midi_to_freq(71)), // B4

        // Middle row - chromatics
        KeyCode::Char('s') => Some(midi_to_freq(61)), // C#4
        KeyCode::Char('d') => Some(midi_to_freq(63)), // D#4
        KeyCode::Char('g') => Some(midi_to_freq(66)), // F#4
        KeyCode::Char('h') => Some(midi_to_freq(68)), // G#4
        KeyCode::Char('j') => Some(midi_to_freq(70)), // A#4

        // Top row - next octave
        KeyCode::Char('q') => Some(midi_to_freq(72)), // C5
        KeyCode::Char('w') => Some(midi_to_freq(74)), // D5
        KeyCode::Char('e') => Some(midi_to_freq(76)), // E5
        KeyCode::Char('r') => Some(midi_to_freq(77)), // F5
        KeyCode::Char('t') => Some(midi_to_freq(79)), // G5
        KeyCode::Char('y') => Some(midi_to_freq(81)), // A5
        KeyCode::Char('u') => Some(midi_to_freq(83)), // B5

        _ => None,
    }
}

/// Convert MIDI note to frequency
fn midi_to_freq(midi_note: u8) -> f64 {
    440.0 * 2.0_f64.powf((midi_note as f64 - 69.0) / 12.0)
}

/// Check if a key represents a musical note
pub fn is_musical_key(key: KeyCode) -> bool {
    key_to_note(key).is_some()
}
