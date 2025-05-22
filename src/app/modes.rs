use crate::app::state::DAWApp;
use crate::modules::ModuleType;
use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    ModuleView,
    ParameterEdit,
    ModuleAdd,
    Connection,
}

impl AppMode {
    pub fn handle_key(&mut self, app: &mut DAWApp, key: KeyEvent) -> bool {
        match self {
            AppMode::ModuleView => self.handle_module_view(app, key),
            AppMode::ParameterEdit => self.handle_parameter_edit(app, key),
            AppMode::ModuleAdd => self.handle_module_add(app, key),
            AppMode::Connection => self.handle_connection(app, key),
        }
    }

    fn handle_module_view(&mut self, app: &mut DAWApp, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') => return true, // Quit
            KeyCode::Tab => {
                app.cycle_selected_module();
            }
            KeyCode::Enter => {
                if app.selected_module.is_some() {
                    *self = AppMode::ParameterEdit;
                    app.selected_parameter = 0;
                }
            }
            KeyCode::Char('a') => {
                *self = AppMode::ModuleAdd;
            }
            KeyCode::Char('c') => {
                if app.selected_module.is_some() {
                    *self = AppMode::Connection;
                }
            }
            KeyCode::Char('d') => {
                // Delete selected module
                if let Some(module_id) = app.selected_module {
                    app.modules.remove(&module_id);
                    app.selected_module = None;
                    // TODO: Remove connections involving this module
                }
            }
            _ => {}
        }
        false
    }

    fn handle_parameter_edit(&mut self, app: &mut DAWApp, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                *self = AppMode::ModuleView;
            }
            KeyCode::Up => {
                if app.selected_parameter > 0 {
                    app.selected_parameter -= 1;
                }
            }
            KeyCode::Down => {
                if let Some(module_id) = app.selected_module {
                    if let Some(module) = app.modules.get(&module_id) {
                        if app.selected_parameter < module.parameters.len().saturating_sub(1) {
                            app.selected_parameter += 1;
                        }
                    }
                }
            }
            KeyCode::Left => {
                app.adjust_selected_parameter(-0.01);
            }
            KeyCode::Right => {
                app.adjust_selected_parameter(0.01);
            }
            KeyCode::Char('-') => {
                app.adjust_selected_parameter(-0.1);
            }
            KeyCode::Char('=') | KeyCode::Char('+') => {
                app.adjust_selected_parameter(0.1);
            }
            _ => {}
        }
        false
    }

    fn handle_module_add(&mut self, app: &mut DAWApp, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                *self = AppMode::ModuleView;
            }
            KeyCode::Char('o') => {
                app.add_module(ModuleType::Oscillator);
                *self = AppMode::ModuleView;
            }
            KeyCode::Char('f') => {
                app.add_module(ModuleType::Filter);
                *self = AppMode::ModuleView;
            }
            KeyCode::Char('r') => {
                app.add_module(ModuleType::Reverb);
                *self = AppMode::ModuleView;
            }
            KeyCode::Char('d') => {
                app.add_module(ModuleType::Delay);
                *self = AppMode::ModuleView;
            }
            KeyCode::Char('u') => {
                app.add_module(ModuleType::Output);
                *self = AppMode::ModuleView;
            }
            KeyCode::Char('s') => {
                app.add_module(ModuleType::Sequencer);
                *self = AppMode::ModuleView;
            }
            _ => {}
        }
        false
    }

    fn handle_connection(&mut self, app: &mut DAWApp, key: KeyEvent) -> bool {
        // TODO: Implement connection logic
        match key.code {
            KeyCode::Esc => {
                *self = AppMode::ModuleView;
            }
            _ => {}
        }
        false
    }

    pub fn get_help_text(&self) -> &'static str {
        match self {
            AppMode::ModuleView => {
                "Tab: Select | Enter: Edit | A: Add | C: Connect | D: Delete | Q: Quit"
            }
            AppMode::ParameterEdit => "↑↓: Select | ←→: Fine | -/+: Coarse | Esc: Back",
            AppMode::ModuleAdd => {
                "O: Osc | F: Filter | R: Reverb | D: Delay | U: Output | S: Seq | Esc: Cancel"
            }
            AppMode::Connection => "Esc: Cancel | (Connection UI coming soon!)",
        }
    }
}
