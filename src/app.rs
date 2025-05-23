use crate::ui::terminal::TerminalUI;
use crate::audio::synth::play_sine_wave;

pub struct App;

impl App {
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let mut ui = TerminalUI::new()?;
        ui.run_loop(|| {
            play_sine_wave(440.0, 2);
        })?;
        Ok(())
    }
}
