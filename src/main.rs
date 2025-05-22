use anyhow::Result;
use crossterm::event::{self, Event};
use maze_daw::app::DAWApp;

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let mut app = DAWApp::new()?;

    let result = loop {
        terminal.draw(|frame| app.draw(frame))?;

        if let Event::Key(key) = event::read()? {
            if app.handle_key(key) {
                break Ok(());
            }
        }
    };

    ratatui::restore();
    result
}
