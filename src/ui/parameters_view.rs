use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{modes::AppMode, DAWApp};

pub fn draw(app: &DAWApp, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("🎚️  Parameters")
        .borders(Borders::ALL)
        .border_style(if matches!(app.mode, AppMode::ParameterEdit) {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Gray)
        });

    if let Some(module_id) = app.selected_module {
        if let Some(module) = app.modules.get(&module_id) {
            draw_parameter_list(app, module, frame, area, block);
        } else {
            let error_msg = Paragraph::new("Selected module not found!")
                .block(block)
                .style(Style::default().fg(Color::Red));
            frame.render_widget(error_msg, area);
        }
    } else {
        let help_msg = Paragraph::new("No module selected\nPress Tab to select a module")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help_msg, area);
    }
}

fn draw_parameter_list(
    app: &DAWApp,
    module: &crate::modules::AudioModule,
    frame: &mut Frame,
    area: Rect,
    block: Block,
) {
    let param_items: Vec<ListItem> = module
        .parameters
        .iter()
        .enumerate()
        .map(|(i, param)| {
            let is_selected =
                i == app.selected_parameter && matches!(app.mode, AppMode::ParameterEdit);

            let style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };

            // Create a visual representation of the parameter value
            let normalized = param.normalized();
            let bar_width = 20;
            let filled_width = (normalized * bar_width as f32) as usize;
            let bar = format!(
                "[{}{}]",
                "█".repeat(filled_width),
                "░".repeat(bar_width - filled_width)
            );

            // Format the value nicely
            let formatted_value = format_parameter_value(param);

            let content = Line::from(vec![
                Span::styled(&param.name, style),
                Span::raw(": "),
                Span::styled(&formatted_value, Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled(
                    &bar,
                    if is_selected {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::Blue)
                    },
                ),
            ]);

            ListItem::new(content).style(style)
        })
        .collect();

    let params_list = List::new(param_items)
        .block(block)
        .highlight_style(Style::default().fg(Color::Black).bg(Color::Green));

    frame.render_widget(params_list, area);

    // Draw additional parameter info if editing
    if matches!(app.mode, AppMode::ParameterEdit) && area.height > 8 {
        draw_parameter_details(app, module, frame, area);
    }
}

fn draw_parameter_details(
    app: &DAWApp,
    module: &crate::modules::AudioModule,
    frame: &mut Frame,
    area: Rect,
) {
    if let Some(param) = module.parameters.get(app.selected_parameter) {
        let detail_area = Rect {
            x: area.x + 2,
            y: area.y + area.height - 3,
            width: area.width - 4,
            height: 2,
        };

        let range_info = format!("Range: {:.2} - {:.2}", param.min, param.max);
        let step_info = format!("Step: {:.3}", param.step);

        let details = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("Range: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &format!("{:.2} - {:.2}", param.min, param.max),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![Span::styled(
                "Use ←→ for fine, -/+ for coarse",
                Style::default().fg(Color::Yellow),
            )]),
        ]);

        frame.render_widget(details, detail_area);
    }
}

fn format_parameter_value(param: &crate::modules::ModuleParameter) -> String {
    match param.name.as_str() {
        "Frequency" | "Cutoff" => {
            if param.value >= 1000.0 {
                format!("{:.1}kHz", param.value / 1000.0)
            } else {
                format!("{:.0}Hz", param.value)
            }
        }
        "Amplitude" | "Volume" | "Wet" | "Resonance" | "Damping" | "Room Size" => {
            format!("{:.0}%", param.value * 100.0)
        }
        "Pan" => {
            let pan_val = (param.value - 0.5) * 200.0;
            if pan_val.abs() < 1.0 {
                "Center".to_string()
            } else if pan_val > 0.0 {
                format!("R{:.0}", pan_val)
            } else {
                format!("L{:.0}", -pan_val)
            }
        }
        "Time" => {
            if param.value >= 1.0 {
                format!("{:.2}s", param.value)
            } else {
                format!("{:.0}ms", param.value * 1000.0)
            }
        }
        "BPM" => format!("{:.0} BPM", param.value),
        "Steps" => format!("{:.0} steps", param.value),
        "Waveform" => match param.value as i32 {
            0 => "Sine".to_string(),
            1 => "Sawtooth".to_string(),
            2 => "Square".to_string(),
            3 => "Triangle".to_string(),
            _ => "Unknown".to_string(),
        },
        "Type" => match param.value as i32 {
            0 => "Lowpass".to_string(),
            1 => "Highpass".to_string(),
            2 => "Bandpass".to_string(),
            _ => "Unknown".to_string(),
        },
        _ => format!("{:.2}", param.value),
    }
}
