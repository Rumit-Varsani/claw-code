//! Command palette component for quick command access

use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Style, Color};
use ratatui::widgets::{Block, Paragraph, List, ListItem, Borders};
use ratatui::text::{Text, Line};

use super::state::FocusState;
use super::styles::Theme;

/// Command palette component
#[derive(Debug)]
pub struct CommandPaletteComponent {
    /// Theme for the component
    pub theme: Theme,

    /// Search query
    pub query: String,

    /// Focused state
    pub focus: FocusState,

    /// Available commands
    pub commands: Vec<CommandItem>,

    /// Selected command index
    pub selected: Option<usize>,
}

/// Command item definition
#[derive(Debug, Clone)]
pub struct CommandItem {
    /// Command identifier
    pub id: CommandId,

    /// Display name
    pub name: String,

    /// Short description
    pub description: String,

    /// Icon (optional)
    pub icon: Option<char>,

    /// Whether this command is available in current context
    pub available: bool,
}

/// Command identifier type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandId {
    /// Send message
    Send,
    /// Clear conversation
    Clear,
    /// Switch to history
    History,
    /// Toggle settings
    Settings,
    /// Toggle help
    Help,
    /// Search conversations
    Search,
    /// Copy message
    Copy,
    /// Clear screen
    ClearScreen,
    /// Quit application
    Quit,
    /// Show status
    Status,
    /// Switch to chat
    Chat,
    /// Execute MCP tool
    ToolExec,
    /// List tools
    ListTools,
    /// Connect remote
    Connect,
    /// Disconnect
    Disconnect,
    /// Change theme
    ToggleTheme,
    /// Format help
    FormatHelp,
}

impl CommandPaletteComponent {
    /// Create a new command palette component
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            query: String::new(),
            focus: FocusState::CommandInput,
            commands: Self::default_commands(),
            selected: None,
        }
    }

    /// Set the search query
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
        self.filter_commands();
        self.selected = if self.commands.is_empty() {
            None
        } else {
            Some(0)
        };
    }

    /// Clear the query
    pub fn clear_query(&mut self) {
        self.query.clear();
        self.filter_commands();
        self.selected = None;
    }

    /// Add a command
    pub fn add_command(&mut self, command: CommandItem) {
        self.commands.push(command);
        self.filter_commands();
    }

    /// Set focus state
    pub fn set_focus(&mut self, focus: FocusState) {
        self.focus = focus;
    }

    /// Set selected command
    pub fn set_selected(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    /// Handle a character input
    pub fn handle_char(&mut self, c: char) {
        if c == '\u{7f}' { // Backspace
            if self.query.len() > 0 {
                self.query.pop();
                self.filter_commands();
                self.selected = if self.commands.is_empty() {
                    None
                } else {
                    Some(0)
                };
            }
        } else if !c.is_control() {
            self.query.push(c);
            self.filter_commands();
            self.selected = if self.commands.is_empty() {
                None
            } else {
                Some(0)
            };
        }
    }

    /// Handle key input
    pub fn handle_key(&mut self, key: ratatui::event::Key) {
        let length = self.commands.len();
        match key {
            ratatui::event::Key::Up => {
                if let Some(idx) = self.selected {
                    self.selected = if idx == 0 {
                        None
                    } else {
                        Some(idx.saturating_sub(1))
                    };
                }
            }
            ratatui::event::Key::Down => {
                let next_idx = self.selected.map_or(0, |idx| idx + 1);
                if next_idx < length {
                    self.selected = Some(next_idx);
                }
            }
            ratatui::event::Key::Enter => {
                if let Some(idx) = self.selected {
                    self.execute_command(&self.commands[idx]);
                }
            }
            ratatui::event::Key::Backspace => {
                if self.query.len() > 0 {
                    self.query.pop();
                    self.filter_commands();
                }
            }
            _ => {}
        }
    }

    /// Execute a selected command
    pub fn execute_command(&mut self, command: &CommandItem) {
        // Handle command execution
        // This would call back to the main application
    }

    /// Render the command palette
    pub fn render(&self, area: Rect, frame: &mut ratatui::Frame) {
        // If query is empty, no display needed
        if self.query.is_empty() {
            return;
        }

        // Build title with query
        let title = format!(": {}", self.query);
        let title_style = self.theme.inactive();

        // Build command list items
        let list_items: Vec<ListItem> = self.commands.iter()
            .enumerate()
            .filter_map(|(idx, cmd)| {
                if self.selected.map_or(true, |s| s == idx) {
                    let text = if cmd.available {
                        format!(
                            "{}  {}  {}",
                            cmd.icon.unwrap_or('•'),
                            cmd.name,
                            cmd.description
                        )
                    } else {
                        format!(
                            "{}  {} (disabled)",
                            cmd.icon.unwrap_or('•'),
                            cmd.name
                        )
                    };
                    let style = self.theme.success();
                    Some(ListItem::new(Text::from(text)).style(style))
                } else {
                    None
                }
            })
            .collect();

        // Get max width for descriptions
        let max_width = if list_items.is_empty() {
            // Show no commands available message
            let msg = vec![
                ListItem::new("No commands found".to_string()).style(self.theme.warning()),
            ];
            self.render_message_list(Vec::new(), Vec::new(), area, frame);
            return;
        } else {
            // Build list for display
            self.render_command_list(list_items.clone(), title, title_style, area, frame);
            return;
        };
    }

    /// Render the command list
    fn render_command_list(
        &self,
        list_items: Vec<ListItem>,
        title: String,
        title_style: Style,
        area: Rect,
        frame: &mut ratatui::Frame,
    ) {
        // Create the list widget
        let list = List::new(list_items)
            .block(Block::default()
                .title("Commands")
                .title_formatter(|line, style| {
                    Text::styled(line, style)
                })
                .borders(Borders::ALL)
                .border_style(self.theme.primary()));

        // Render with selected index highlight
        let mut state = Cursor::new(self.selected.unwrap_or(0));
        frame.render_stateful_widget(list, area, &mut state);
    }

    /// Render a message when no commands are found
    fn render_message_list(
        &self,
        items: Vec<(String, Style)>,
        title: Vec<Line>,
        area: Rect,
        frame: &mut ratatui::Frame,
    ) {
        let content = items.iter()
            .map(|(text, style)| Text::styled(text, *style))
            .collect();

        let paragraph = Paragraph::new(content)
            .block(Block::default()
                .title("Commands")
                .borders(Borders::ALL)
                .border_style(self.theme.warning()));

        frame.render_widget(paragraph, area);
    }

    /// Filter commands by query
    fn filter_commands(&mut self) {
        let query_lower = self.query.to_lowercase();
        self.commands.retain(|cmd| {
            cmd.available && (
                cmd.name.to_lowercase().contains(&query_lower) ||
                cmd.description.to_lowercase().contains(&query_lower)
            )
        });
    }

    /// Get default commands
    fn default_commands() -> Vec<CommandItem> {
        vec![
            CommandItem {
                id: CommandId::Send,
                name: "Send Message".to_string(),
                description: "Send your message to the model".to_string(),
                icon: Some('s'),
                available: true,
            },
            CommandItem {
                id: CommandId::Clear,
                name: "Clear Screen".to_string(),
                description: "Clear the conversation history".to_string(),
                icon: Some('c'),
                available: true,
            },
            CommandItem {
                id: CommandId::History,
                name: "History".to_string(),
                description: "Open conversation history view".to_string(),
                icon: Some('h'),
                available: true,
            },
            CommandItem {
                id: CommandId::Settings,
                name: "Settings".to_string(),
                description: "Configure application settings".to_string(),
                icon: Some('S'),
                available: true,
            },
            CommandItem {
                id: CommandId::Help,
                name: "Help".to_string(),
                description: "Show keyboard shortcuts".to_string(),
                icon: Some('?'),
                available: true,
            },
            CommandItem {
                id: CommandId::Status,
                name: "Status".to_string(),
                description: "Show system status".to_string(),
                icon: Some('i'),
                available: true,
            },
            CommandItem {
                id: CommandId::Quit,
                name: "Quit".to_string(),
                description: "Exit the application".to_string(),
                icon: Some('q'),
                available: true,
            },
        ]
    }
}

impl Default for CommandPaletteComponent {
    fn default() -> Self {
        Self::new(Theme::default())
    }
}

/// Simple cursor state
struct Cursor {
    index: usize,
}

impl Cursor {
    fn new(index: usize) -> Self {
        Self { index }
    }
}

impl ratatui::widgets::StatefulWidget for Cursor {
    type State = usize;

    fn render(
        self,
        state: &mut Self::State,
        area: Rect,
        buf: &mut ratatui::buffer::Buffer,
    ) {
        ratatui::widgets::List::new(vec![ListItem::new("")])
            .highlight_style(Style::default().bg(Color::Cyan))
            .highlight_symbol("◆ ")
            .render(
                if *state > 0 {
                    Rect {
                        x: area.x,
                        y: area.y + *state as u16,
                        width: area.width,
                        height: 1,
                    }
                } else {
                    area
                },
                buf,
            );
    }

    fn with_focused_state<F>(self, _focused: bool, _f: F)
    where
        F: FnOnce(&mut Self),
    {
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new(0)
    }
}