mod app;
mod audio;
mod ui;

use app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::run()
}
