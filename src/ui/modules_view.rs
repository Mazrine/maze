use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::DAWApp;
use crate::modules::ModuleType;

pub fn draw(app: &DAWApp, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("🎛️  Modules")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    if app.modules.is_empty() {
        let empty_msg = Paragraph::new("No modules loaded\nPress 'A' to add modules")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty_msg, area);
        return;
    }

    let module_items: Vec<ListItem> = app
        .modules
        .values()
        .map(|module| {
            let is_selected = Some(module.id) == app.selected_module;

            let style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Yellow)
            } else if !module.enabled {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            let type_icon = match module.module_type {
                ModuleType::Oscillator => "🌊",
                ModuleType::Filter => "🔧",
                ModuleType::Reverb => "🏛️",
                ModuleType::Delay => "⏰",
                ModuleType::Output => "🔊",
                ModuleType::Sequencer => "🎹",
            };

            let type_short = match module.module_type {
                ModuleType::Oscillator => "OSC",
                ModuleType::Filter => "FLT",
                ModuleType::Reverb => "REV",
                ModuleType::Delay => "DLY",
                ModuleType::Output => "OUT",
                ModuleType::Sequencer => "SEQ",
            };

            // Show some key parameter info
            let param_info = match module.module_type {
                ModuleType::Oscillator => {
                    let freq = module.get_parameter_value("Frequency");
                    let amp = module.get_parameter_value("Amplitude");
                    format!("{}Hz {:.1}%", freq as i32, amp * 100.0)
                }
                ModuleType::Filter => {
                    let cutoff = module.get_parameter_value("Cutoff");
                    let res = module.get_parameter_value("Resonance");
                    format!("{}Hz Q{:.1}", cutoff as i32, res)
                }
                ModuleType::Output => {
                    let vol = module.get_parameter_value("Volume");
                    let pan = module.get_parameter_value("Pan");
                    format!("Vol{:.0}% Pan{:.0}", vol * 100.0, (pan - 0.5) * 200.0)
                }
                _ => String::new(),
            };

            let content = if param_info.is_empty() {
                Line::from(vec![
                    Span::raw(type_icon),
                    Span::raw(" "),
                    Span::styled(&module.name, style),
                    Span::raw(" ("),
                    Span::styled(type_short, Style::default().fg(Color::Gray)),
                    Span::raw(")"),
                ])
            } else {
                Line::from(vec![
                    Span::raw(type_icon),
                    Span::raw(" "),
                    Span::styled(&module.name, style),
                    Span::raw(" "),
                    Span::styled(&param_info, Style::default().fg(Color::Green)),
                ])
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let modules_list = List::new(module_items)
        .block(block)
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow));

    frame.render_widget(modules_list, area);

    // Show connection info if we have space
    draw_connection_hints(app, frame, area);
}

fn draw_connection_hints(app: &DAWApp, frame: &mut Frame, area: Rect) {
    if area.height < 10 || app.connections.is_empty() {
        return;
    }

    // TODO: Draw connection visualization when we implement the connection system
    // For now, just show connection count
    let connection_count = app.connections.len();
    if connection_count > 0 {
        let hint_area = Rect {
            x: area.x + 2,
            y: area.y + area.height - 2,
            width: area.width - 4,
            height: 1,
        };

        let hint = Paragraph::new(format!("🔗 {} connections", connection_count))
            .style(Style::default().fg(Color::Green));
        frame.render_widget(hint, hint_area);
    }
}
