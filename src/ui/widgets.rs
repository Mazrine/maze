use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::DAWApp;

pub fn draw_controls(app: &DAWApp, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(area);

    draw_help_panel(app, frame, chunks[0]);
    draw_status_bar(app, frame, chunks[1]);
}

fn draw_help_panel(app: &DAWApp, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("🎮 Controls")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let help_text = app.mode.get_help_text();

    // Split help text into lines and style them
    let help_lines: Vec<Line> = help_text
        .split(" | ")
        .map(|section| {
            if let Some((key, desc)) = section.split_once(": ") {
                Line::from(vec![
                    Span::styled(key, Style::default().fg(Color::Yellow)),
                    Span::raw(": "),
                    Span::styled(desc, Style::default().fg(Color::White)),
                ])
            } else {
                Line::from(Span::styled(section, Style::default().fg(Color::Gray)))
            }
        })
        .collect();

    let paragraph = Paragraph::new(help_lines)
        .block(block)
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, area);
}

fn draw_status_bar(app: &DAWApp, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("📊 Status")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    let module_count = app.modules.len();
    let connection_count = app.connections.len();
    let selected_name = app
        .selected_module
        .and_then(|id| app.modules.get(&id))
        .map(|m| m.name.as_str())
        .unwrap_or("None");

    let status_lines = vec![
        Line::from(vec![
            Span::styled("Modules: ", Style::default().fg(Color::Gray)),
            Span::styled(&module_count.to_string(), Style::default().fg(Color::Green)),
            Span::raw("  "),
            Span::styled("Connections: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &connection_count.to_string(),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::styled("Selected: ", Style::default().fg(Color::Gray)),
            Span::styled(selected_name, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Mode: ", Style::default().fg(Color::Gray)),
            Span::styled(&format!("{:?}", app.mode), Style::default().fg(Color::Cyan)),
        ]),
    ];

    let paragraph = Paragraph::new(status_lines)
        .block(block)
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, area);
}

pub fn draw_connection_modal(frame: &mut Frame, area: Rect) {
    // TODO: Implement connection modal when we add connection functionality
    let block = Block::default()
        .title("🔗 Connect Modules")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let help_text = "Connection interface coming soon!\nPress Esc to cancel";

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .alignment(Alignment::Center);

    // Center the modal on screen
    let modal_area = centered_rect(60, 40, area);
    frame.render_widget(paragraph, modal_area);
}

/// Helper function to create a centered rect
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
