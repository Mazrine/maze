// src/ui/terminal.rs
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
};
use std::io::{self, Stdout};
use std::sync::{Arc, Mutex};
use std::time::Duration; // Needed for shared state

pub struct TerminalUI {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    debug_messages: Arc<Mutex<Vec<String>>>, // Hold a reference to the shared messages
}

impl TerminalUI {
    // Modify new to accept the shared debug_messages
    pub fn new(
        debug_messages: Arc<Mutex<Vec<String>>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        execute!(stdout, Clear(ClearType::All))?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self {
            terminal,
            debug_messages,
        }) // Store the Arc
    }

    pub fn run_loop<F>(&mut self, mut play_callback: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(),
    {
        loop {
            self.terminal.draw(|f| {
                let overall_area = f.area();

                let main_layout_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                    .split(overall_area);

                let main_block_area = main_layout_chunks[0];
                let info_section_area = main_layout_chunks[1];

                // --- Main Block (Left Side) ---
                let main_block = Block::default()
                    .title(" Main Controls ")
                    .borders(Borders::ALL);
                f.render_widget(main_block, main_block_area);

                let inner_main_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                    .split(main_block_area);

                let paragraph =
                    Paragraph::new("Press SPACE to play 440Hz sine wave.\nPress 'q' to quit.")
                        .style(
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        );
                f.render_widget(paragraph, inner_main_chunks[0]);

                // --- Info Section (Right Side) ---
                let info_layout_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(info_section_area);

                let selected_info_area = info_layout_chunks[0];
                let debug_block_area = info_layout_chunks[1];

                // Render Selected Info Block
                let selected_info_block = Block::default()
                    .title(" Selected Info ")
                    .borders(Borders::ALL);
                f.render_widget(selected_info_block, selected_info_area);

                let inner_selected_info_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Min(0)].as_ref())
                    .split(selected_info_area);
                let selected_info_paragraph = Paragraph::new("Current: Sine Wave (440Hz)")
                    .style(Style::default().fg(Color::Green));
                f.render_widget(selected_info_paragraph, inner_selected_info_chunks[0]);

                // Render Debug Block
                let debug_block = Block::default().title(" Debug Info ").borders(Borders::ALL);
                f.render_widget(debug_block, debug_block_area);

                // --- IMPORTANT: Render debug messages here ---
                if let Ok(messages) = self.debug_messages.lock() {
                    let debug_text = messages.join("\n");
                    let debug_paragraph =
                        Paragraph::new(debug_text).style(Style::default().fg(Color::Yellow));
                    let inner_debug_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints([Constraint::Min(0)].as_ref())
                        .split(debug_block_area);
                    f.render_widget(debug_paragraph, inner_debug_chunks[0]);
                } else {
                    // Handle case where mutex is poisoned (e.g., panic in another thread)
                    let debug_paragraph = Paragraph::new("Error: Debug message buffer locked!")
                        .style(Style::default().fg(Color::Red));
                    let inner_debug_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints([Constraint::Min(0)].as_ref())
                        .split(debug_block_area);
                    f.render_widget(debug_paragraph, inner_debug_chunks[0]);
                }
                // --- End debug message rendering ---
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
        Ok(())
    }
}

impl Drop for TerminalUI {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}
