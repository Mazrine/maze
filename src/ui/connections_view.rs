use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

use crate::app::DAWApp;
use crate::modules::{Connection, ModuleType};

/// Draw the connection interface
pub fn draw(app: &DAWApp, frame: &mut Frame, area: Rect) {
    // For now, show a simple connection overview
    draw_connection_overview(app, frame, area);
}

/// Draw an overview of all connections in the system
fn draw_connection_overview(app: &DAWApp, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("🔗 Connections")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    if app.connections.is_empty() {
        let empty_msg = Paragraph::new("No connections yet\nPress 'C' to connect modules")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty_msg, area);
        return;
    }

    let connection_items: Vec<ListItem> = app
        .connections
        .iter()
        .enumerate()
        .map(|(i, conn)| {
            let from_module = app.modules.get(&conn.from_module);
            let to_module = app.modules.get(&conn.to_module);

            let from_name = from_module.map(|m| m.name.as_str()).unwrap_or("???");
            let to_name = to_module.map(|m| m.name.as_str()).unwrap_or("???");

            let from_icon = from_module.map(get_module_icon).unwrap_or("❓");
            let to_icon = to_module.map(get_module_icon).unwrap_or("❓");

            let content = Line::from(vec![
                Span::styled(format!("{}. ", i + 1), Style::default().fg(Color::Gray)),
                Span::raw(from_icon),
                Span::styled(from_name, Style::default().fg(Color::Cyan)),
                Span::styled(
                    &format!("[{}]", conn.from_output),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(" ──► ", Style::default().fg(Color::Yellow)),
                Span::raw(to_icon),
                Span::styled(to_name, Style::default().fg(Color::Cyan)),
                Span::styled(
                    &format!("[{}]", conn.to_input),
                    Style::default().fg(Color::Red),
                ),
            ]);

            ListItem::new(content)
        })
        .collect();

    let connections_list = List::new(connection_items).block(block);

    frame.render_widget(connections_list, area);
}

/// Draw the connection creation modal
pub fn draw_connection_modal(app: &DAWApp, frame: &mut Frame) {
    let area = frame.area();

    // Create a centered modal
    let modal_area = centered_rect(60, 40, area);

    // Clear the background
    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title("🔌 Connect Modules")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    if let Some(selected_id) = app.selected_module {
        if let Some(selected_module) = app.modules.get(&selected_id) {
            draw_connection_interface(app, frame, modal_area, selected_module);
        } else {
            let error = Paragraph::new("Selected module not found!")
                .block(block)
                .style(Style::default().fg(Color::Red));
            frame.render_widget(error, modal_area);
        }
    } else {
        let help = Paragraph::new("No module selected\nSelect a module first, then press 'C'")
            .block(block)
            .style(Style::default().fg(Color::Gray));
        frame.render_widget(help, modal_area);
    }
}

/// Draw the detailed connection interface for a specific module
fn draw_connection_interface(
    app: &DAWApp,
    frame: &mut Frame,
    area: Rect,
    selected_module: &crate::modules::AudioModule,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Main content
            Constraint::Length(3), // Instructions
        ])
        .split(area);

    // Header
    let header = Paragraph::new(format!(
        "Connecting: {} {} ({})",
        get_module_icon(selected_module),
        selected_module.name,
        get_module_type_name(selected_module.module_type)
    ))
    .block(Block::default().borders(Borders::ALL))
    .style(Style::default().fg(Color::Yellow));
    frame.render_widget(header, chunks[0]);

    // Main content - split into inputs and outputs
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    draw_module_inputs(app, frame, content_chunks[0], selected_module);
    draw_module_outputs(app, frame, content_chunks[1], selected_module);

    // Instructions
    let instructions =
        Paragraph::new("Use arrow keys to navigate, Enter to connect, Esc to cancel")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray));
    frame.render_widget(instructions, chunks[2]);
}

/// Draw the inputs section of the connection interface
fn draw_module_inputs(
    app: &DAWApp,
    frame: &mut Frame,
    area: Rect,
    module: &crate::modules::AudioModule,
) {
    let block = Block::default()
        .title("📥 Inputs")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    if module.input_count == 0 {
        let no_inputs = Paragraph::new("No inputs available")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(no_inputs, area);
        return;
    }

    let input_items: Vec<ListItem> = (0..module.input_count)
        .map(|i| {
            // Find if this input is connected
            let connected_from = app
                .connections
                .iter()
                .find(|conn| conn.to_module == module.id && conn.to_input == i)
                .and_then(|conn| app.modules.get(&conn.from_module));

            let content = if let Some(from_module) = connected_from {
                Line::from(vec![
                    Span::styled(&format!("In {}: ", i), Style::default().fg(Color::Red)),
                    Span::raw(get_module_icon(from_module)),
                    Span::styled(&from_module.name, Style::default().fg(Color::Green)),
                ])
            } else {
                Line::from(vec![
                    Span::styled(&format!("In {}: ", i), Style::default().fg(Color::Red)),
                    Span::styled("(empty)", Style::default().fg(Color::DarkGray)),
                ])
            };

            ListItem::new(content)
        })
        .collect();

    let inputs_list = List::new(input_items).block(block);
    frame.render_widget(inputs_list, area);
}

/// Draw the outputs section of the connection interface
fn draw_module_outputs(
    app: &DAWApp,
    frame: &mut Frame,
    area: Rect,
    module: &crate::modules::AudioModule,
) {
    let block = Block::default()
        .title("📤 Outputs")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    if module.output_count == 0 {
        let no_outputs = Paragraph::new("No outputs available")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(no_outputs, area);
        return;
    }

    let output_items: Vec<ListItem> = (0..module.output_count)
        .map(|i| {
            // Find all modules connected to this output
            let connected_to: Vec<_> = app
                .connections
                .iter()
                .filter(|conn| conn.from_module == module.id && conn.from_output == i)
                .filter_map(|conn| app.modules.get(&conn.to_module))
                .collect();

            let content = if connected_to.is_empty() {
                Line::from(vec![
                    Span::styled(&format!("Out {}: ", i), Style::default().fg(Color::Green)),
                    Span::styled("(empty)", Style::default().fg(Color::DarkGray)),
                ])
            } else if connected_to.len() == 1 {
                Line::from(vec![
                    Span::styled(&format!("Out {}: ", i), Style::default().fg(Color::Green)),
                    Span::raw(get_module_icon(connected_to[0])),
                    Span::styled(&connected_to[0].name, Style::default().fg(Color::Cyan)),
                ])
            } else {
                Line::from(vec![
                    Span::styled(&format!("Out {}: ", i), Style::default().fg(Color::Green)),
                    Span::styled(
                        &format!("{} modules", connected_to.len()),
                        Style::default().fg(Color::Yellow),
                    ),
                ])
            };

            ListItem::new(content)
        })
        .collect();

    let outputs_list = List::new(output_items).block(block);
    frame.render_widget(outputs_list, area);
}

/// Draw a visual connection graph (simplified)
pub fn draw_connection_graph(app: &DAWApp, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title("🕸️  Connection Graph")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    if app.modules.is_empty() {
        let empty = Paragraph::new("No modules to display")
            .block(block)
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
        return;
    }

    // Simple text-based graph representation
    let graph_lines: Vec<Line> = app
        .modules
        .values()
        .enumerate()
        .map(|(i, module)| {
            let connected_outputs: Vec<_> = app
                .connections
                .iter()
                .filter(|conn| conn.from_module == module.id)
                .filter_map(|conn| app.modules.get(&conn.to_module))
                .collect();

            let module_display = format!("{} {}", get_module_icon(module), module.name);

            if connected_outputs.is_empty() {
                Line::from(vec![
                    Span::styled(&format!("{}. ", i + 1), Style::default().fg(Color::Gray)),
                    Span::styled(&module_display, Style::default().fg(Color::White)),
                ])
            } else {
                let connections_text = connected_outputs
                    .iter()
                    .map(|m| format!("{} {}", get_module_icon(m), m.name))
                    .collect::<Vec<_>>()
                    .join(", ");

                Line::from(vec![
                    Span::styled(&format!("{}. ", i + 1), Style::default().fg(Color::Gray)),
                    Span::styled(&module_display, Style::default().fg(Color::White)),
                    Span::styled(" ──► ", Style::default().fg(Color::Yellow)),
                    Span::styled(&connections_text, Style::default().fg(Color::Cyan)),
                ])
            }
        })
        .collect();

    let graph = Paragraph::new(graph_lines)
        .block(block)
        .wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(graph, area);
}

/// Get an icon representing the module type
fn get_module_icon(module: &crate::modules::AudioModule) -> &'static str {
    match module.module_type {
        ModuleType::Oscillator => "🌊",
        ModuleType::Filter => "🔧",
        ModuleType::Reverb => "🏛️",
        ModuleType::Delay => "⏰",
        ModuleType::Output => "🔊",
        ModuleType::Sequencer => "🎹",
    }
}

/// Get a human-readable name for the module type
fn get_module_type_name(module_type: ModuleType) -> &'static str {
    match module_type {
        ModuleType::Oscillator => "Oscillator",
        ModuleType::Filter => "Filter",
        ModuleType::Reverb => "Reverb",
        ModuleType::Delay => "Delay",
        ModuleType::Output => "Output",
        ModuleType::Sequencer => "Sequencer",
    }
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

/// Draw connection hints and tips
pub fn draw_connection_help(frame: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from("Connection Tips:"),
        Line::from(""),
        Line::from("• Oscillators generate audio signals"),
        Line::from("• Filters process audio signals"),
        Line::from("• Effects (reverb, delay) add ambience"),
        Line::from("• Outputs send audio to speakers"),
        Line::from("• Sequencers trigger other modules"),
        Line::from(""),
        Line::from("Typical signal flow:"),
        Line::from("🎹 Sequencer ──► 🌊 Oscillator ──► 🔧 Filter ──► 🏛️ Reverb ──► 🔊 Output"),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title("📚 Help")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(help, area);
}

/// Validate if a connection would be valid
pub fn validate_connection(
    app: &DAWApp,
    from_module: uuid::Uuid,
    from_output: usize,
    to_module: uuid::Uuid,
    to_input: usize,
) -> Result<(), String> {
    // Check if modules exist
    let from = app
        .modules
        .get(&from_module)
        .ok_or("Source module not found")?;
    let to = app
        .modules
        .get(&to_module)
        .ok_or("Destination module not found")?;

    // Check if trying to connect to self
    if from_module == to_module {
        return Err("Cannot connect module to itself".to_string());
    }

    // Check output index
    if from_output >= from.output_count {
        return Err("Invalid output index".to_string());
    }

    // Check input index
    if to_input >= to.input_count {
        return Err("Invalid input index".to_string());
    }

    // Check if connection already exists
    if app.connections.iter().any(|conn| {
        conn.from_module == from_module
            && conn.from_output == from_output
            && conn.to_module == to_module
            && conn.to_input == to_input
    }) {
        return Err("Connection already exists".to_string());
    }

    // Check if input is already occupied
    if app
        .connections
        .iter()
        .any(|conn| conn.to_module == to_module && conn.to_input == to_input)
    {
        return Err("Input already connected".to_string());
    }

    Ok(())
}
