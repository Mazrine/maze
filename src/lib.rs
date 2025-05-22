pub mod app;
pub mod audio;
pub mod modules;
pub mod ui;
pub mod utils;

// Re-export commonly used types for convenience
pub use app::DAWApp;
pub use modules::types::{AudioModule, ModuleType, ModuleParameter};
pub use audio::engine::AudioEngine;
