// src/main.rs
mod app;
mod audio;
mod ui;

// App::run() now handles initialization.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    app::App::new()?.run()
}
