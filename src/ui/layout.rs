use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use super::{modules_view, parameters_view, widgets};
use crate::app::DAWApp;

pub fn draw_main_layout(app: &DAWApp, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(frame.area());

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    // Draw main sections
    modules_view::draw(app, frame, chunks[0]);
    parameters_view::draw(app, frame, right_chunks[0]);
    widgets::draw_controls(app, frame, right_chunks[1]);
}
