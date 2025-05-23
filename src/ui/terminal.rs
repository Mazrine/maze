use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};
use std::io::{self, Stdout};
use std::time::Duration; // Add this import for execute!

pub struct TerminalUI {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalUI {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // 1. Enable raw mode
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        // 2. Enter alternate screen buffer
        execute!(stdout, EnterAlternateScreen)?; // <--- ADD THIS LINE
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn run_loop<F>(&mut self, mut play_callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(),
    {
        // Your existing loop code remains the same
        loop {
            self.terminal.draw(|f| {
                let area = f.area();
                let block = Block::default().title("Maze Synth").borders(Borders::ALL);
                f.render_widget(block, area);

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(5)
                    .constraints([Constraint::Length(3)].as_ref())
                    .split(area);

                let paragraph =
                    Paragraph::new("Press SPACE to play 440Hz sine wave.\nPress 'q' to quit.")
                        .style(
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        );
                f.render_widget(paragraph, chunks[0]);
            })?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char(' ') => play_callback(),
                        _ => {}
                    }
                }
            }
        }
        // The cleanup is now handled by the Drop implementation
        // so you can remove these two lines from run_loop:
        // crossterm::terminal::disable_raw_mode()?;
        // self.terminal.show_cursor()?;
        //
        // However, if run_loop needs to explicitly ensure cleanup
        // (e.g., if you don't want to rely solely on Drop for some reason),
        // you would call a cleanup method here.
        Ok(())
    }
}

// Add a Drop implementation to ensure terminal is restored on exit/error
impl Drop for TerminalUI {
    fn drop(&mut self) {
        // Attempt to restore the terminal state
        let _ = disable_raw_mode(); // Ignore error as we're exiting
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen); // Ignore error
        let _ = self.terminal.show_cursor(); // Ignore error
    }
}
