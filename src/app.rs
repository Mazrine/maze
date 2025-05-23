// src/app.rs
use crate::audio::synth::play_sine_wave;
use crate::ui::terminal::TerminalUI;
use std::sync::{Arc, Mutex};
// Add #[allow(unused_imports)] to the module to suppress this specific warning
#[allow(unused_imports)]
use log::{LevelFilter, error, info, warn}; // Keep error and info

// This struct will hold all application-wide state.
pub struct App {
    ui: TerminalUI,
    #[allow(dead_code)] // Keep this to suppress the 'field never read' warning
    pub debug_messages: Arc<Mutex<Vec<String>>>,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let debug_messages = Arc::new(Mutex::new(Vec::new()));

        log::set_max_level(LevelFilter::Info);

        if let Err(e) = log::set_logger(Box::leak(TuiLogger::new(Arc::clone(&debug_messages)))) {
            error!("Failed to set logger: {}", e); // This uses the 'error' import
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to set logger: {}", e),
            )));
        }

        let ui = TerminalUI::new(Arc::clone(&debug_messages))?;
        Ok(Self { ui, debug_messages })
    }

    pub fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Application started.");

        self.ui.run_loop(|| {
            info!("Attempting to play sine wave...");
            play_sine_wave(440.0, 2);
            info!("Sine wave played for 2 seconds.");
        })?;

        info!("Application gracefully shut down.");
        Ok(())
    }
}

// Define the custom logger for the TUI
pub struct TuiLogger {
    messages: Arc<Mutex<Vec<String>>>,
}

impl TuiLogger {
    pub fn new(messages: Arc<Mutex<Vec<String>>>) -> Box<Self> {
        Box::new(TuiLogger { messages })
    }
}

impl log::Log for TuiLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let message = format!("{} [{}] {}", record.level(), record.target(), record.args());
            if let Ok(mut messages_guard) = self.messages.lock() {
                const MAX_MESSAGES: usize = 20;
                let current_len = messages_guard.len();
                if current_len >= MAX_MESSAGES {
                    messages_guard.drain(0..current_len - MAX_MESSAGES + 1);
                }
                messages_guard.push(message);
            } else {
                // This 'warn' is implicitly used here if the mutex is poisoned.
                // We're using eprintln! as a fallback in this critical scenario.
                eprintln!(
                    "WARNING: Failed to acquire debug messages lock for logging: {}",
                    message
                );
            }
        }
    }

    fn flush(&self) {
        // No-op for this logger, as messages are pushed immediately
    }
}
