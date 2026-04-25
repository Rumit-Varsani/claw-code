//! UI components for displaying messages

use ratatui::layout::{Rect, Constraint};
use ratatui::style::{Style, Color};
use ratatui::widgets::{Block, Paragraph, List, ListItem, Borders, Wrap};
use ratatui::text::{Text, Line};

use super::state::{CursorPosition, MessageFilter};
use super::styles::Theme;

/// Message component for displaying chat messages
#[derive(Debug)]
pub struct MessageComponent {
    /// Theme for the component
    pub theme: Theme,

    /// Display filter for messages
    pub filter: MessageFilter,

    /// Cursor position
    pub cursor: CursorPosition,

    /// Scroll offset
    pub scroll_offset: usize,

    /// Messages to display
    pub messages: Vec<MessageItem>,
}

#[derive(Debug, Clone)]
pub struct MessageItem {
    /// Message role (user, assistant, system)
    pub role: MessageRole,

    /// Message content
    pub content: String,

    /// Timestamp (optional)
    pub timestamp: Option<String>,

    /// Whether this message is unread
    pub is_new: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Error,
}

impl MessageComponent {
    /// Create a new message component
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            filter: MessageFilter::All,
            cursor: CursorPosition::Bottom,
            scroll_offset: 0,
            messages: Vec::new(),
        }
    }

    /// Add a user message
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.messages.push(MessageItem {
            role: MessageRole::User,
            content: content.into(),
            timestamp: None,
            is_new: true,
        });
        self.cursor = CursorPosition::Bottom;
    }

    /// Add an assistant response
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.messages.push(MessageItem {
            role: MessageRole::Assistant,
            content: content.into(),
            timestamp: None,
            is_new: true,
        });
        self.cursor = CursorPosition::Bottom;
    }

    /// Add a system message
    pub fn add_system_message(&mut self, content: impl Into<String>) {
        self.messages.push(MessageItem {
            role: MessageRole::System,
            content: content.into(),
            timestamp: None,
            is_new: true,
        });
        self.cursor = CursorPosition::Top;
    }

    /// Add an error message
    pub fn add_error_message(&mut self, content: impl Into<String>) {
        self.messages.push(MessageItem {
            role: MessageRole::Error,
            content: content.into(),
            timestamp: None,
            is_new: true,
        });
        self.cursor = CursorPosition::Bottom;
    }

    /// Set the filter for displaying messages
    pub fn set_filter(&mut self, filter: MessageFilter) {
        self.filter = filter;
    }

    /// Update cursor position
    pub fn set_cursor(&mut self, position: CursorPosition) {
        self.cursor = position;
    }

    /// Get the current scroll offset
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Increment scroll offset
    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(1);
        }
    }

    /// Decrement scroll offset
    pub fn scroll_down(&mut self) {
        if self.scroll_offset < self.len() {
            self.scroll_offset += 1;
        }
    }

    /// Scroll to top
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    /// Scroll to bottom
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = self.len().saturating_sub(1);
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Get the number of messages
    pub fn len(&self) -> usize {
        let filtered = self.filtered_messages().len();
        filtered
    }

    /// Check if there are any messages
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Get messages matching the current filter
    fn filtered_messages(&self) -> &[MessageItem] {
        match self.filter {
            MessageFilter::All => &self.messages,
            MessageFilter::UserOnly => self.messages.iter().filter(|m| m.role == MessageRole::User).collect::<Vec<_>>().as_slice(),
            MessageFilter::AssistantOnly => self.messages.iter().filter(|m| m.role == MessageRole::Assistant).collect::<Vec<_>>().as_slice(),
            MessageFilter::Errors => self.messages.iter().filter(|m| m.role == MessageRole::Error).collect::<Vec<_>>().as_slice(),
        }
    }

    /// Render the message component
    pub fn render(&self, area: Rect, frame: &mut ratatui::Frame) {
        // Build the content
        let content = self.build_content();

        // Create a paragraph widget
        let paragraphs = Paragraph::new(content)
            .block(Block::default()
                .title("Messages")
                .borders(Borders::ALL)
                .border_style(self.theme.primary()))
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraphs, area);
    }

    /// Build the text content for rendering
    fn build_content(&self) -> Text<'_> {
        let filtered = self.filtered_messages();
        let total_lines = filtered.len();

        // Apply scroll offset
        let display_messages: Vec<&MessageItem> = if total_lines <= self.scroll_offset {
            FilteredIter::new(filtered, total_lines, 0)
        } else {
            FilteredIter::new(filtered, total_lines, self.scroll_offset)
        }.map(|&(i, _)| i)
            .collect();

        // Build lines
        let mut lines = Vec::new();

        for message in display_messages {
            // Add role label
            let role_text = match message.role {
                MessageRole::User => "[User]",
                MessageRole::Assistant => "[Assistant]",
                MessageRole::System => "[System]",
                MessageRole::Error => "[Error]",
            };

            let style = match message.role {
                MessageRole::User => self.theme.primary(),
                MessageRole::Assistant => self.theme.success(),
                MessageRole::System | MessageRole::Error => self.theme.warning(),
            };

            // Add line with role and content
            let mut line = Line::from(role_text.to_string()).style(style);
            line.extend(
                textwrap::wrap(&message.content, area.width.saturating_sub(10))
                    .into_iter()
                    .map(|text| Line::from(text))
            );
            lines.push(line);
        }

        Text::from(lines)
    }
}

/// Iterator for filtered and offset messages
struct FilteredIter<'a> {
    items: Vec<usize>,
    index: usize,
}

impl<'a> FilteredIter<'a> {
    fn new(items: &'a [MessageItem], total: usize, offset: usize) -> Self {
        let indices: Vec<usize> = items.iter().enumerate().map(|(i, m)| (i, m.is_new)).collect();

        // Filter out read messages only for offset calculation
        // This simulates scrolling behavior - new messages stay at bottom
        let mut idx = if total > offset {
            total.saturating_sub(offset)
        } else {
            0
        };

        let items: Vec<usize> = items.iter().enumerate()
            .rev()
            .skip(idx.saturating_sub(1))
            .take(offset + 1)
            .map(|(i, _)| i)
            .collect();

        Self { items, index: 0 }
    }
}

impl<'a> Iterator for FilteredIter<'a> {
    type Item = (&'a MessageItem, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.items.len() {
            let idx = self.items[self.index];
            self.index += 1;
            Some((&self.items[idx], idx))
        } else {
            None
        }
    }
}

/// Message list component for history view
pub struct MessageListComponent {
    /// Theme for the component
    pub theme: Theme,

    /// Selected message index
    pub selected: Option<usize>,

    /// Messages to display
    pub messages: Vec<ConversationListItem>,
}

#[derive(Debug, Clone)]
pub struct ConversationListItem {
    /// Conversation ID
    pub id: String,

    /// Title or first message preview
    pub title: String,

    /// Message count
    pub message_count: usize,

    /// Timestamp
    pub timestamp: String,

    /// Whether this is a new conversation
    pub is_new: bool,
}

impl MessageListComponent {
    /// Create a new message list component
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            selected: None,
            messages: Vec::new(),
        }
    }

    /// Add a conversation to the list
    pub fn add_conversation(&mut self, item: ConversationListItem) {
        self.messages.push(item);
    }

    /// Set the selected conversation
    pub fn set_selected(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    /// Render the list component
    pub fn render(&self, area: Rect, frame: &mut ratatui::Frame) {
        // Get list items
        let list_items = self.build_list_items();

        // Create the list widget
        let list = List::new(list_items)
            .block(Block::default()
                .title("Conversations")
                .borders(Borders::ALL)
                .border_style(self.theme.primary()))
            .highlight_style(Style::default()
                .bg(self.theme.primary().fg))
            .highlight_symbol("> ");

        // Render with or without selection context
        if let Some(idx) = self.selected {
            frame.render_stateful_widget(list, area, &mut Cursor::new(idx));
        } else {
            frame.render_widget(list, area);
        }
    }

    /// Get the number of conversations
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Build ListItem from conversation items
    fn build_list_items(&self) -> Vec<ListItem> {
        self.messages.iter()
            .enumerate()
            .map(|(i, item)| {
                let text = format!(
                    "{} ({} {}) | {}",
                    item.title,
                    item.message_count,
                    if item.message_count == 1 { "msg" } else { "msgs" },
                    item.timestamp
                );
                let style = if Some(i) == self.selected {
                    Style::default().bg(self.theme.primary().fg)
                } else {
                    Style::default()
                };

                ListItem::new(text).style(style)
            })
            .collect()
    }
}

/// Simple scrollable cursor state
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
            .highlight_style(Style::default().bg(Color::Rgb(80, 80, 200)))
            .highlight_symbol("► ")
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

/// Input component for text entry
pub struct InputComponent {
    /// Theme for the component
    pub theme: Theme,

    /// Input text
    pub value: String,

    /// Placeholder text
    pub placeholder: String,

    /// Is input focused
    pub focused: bool,

    /// Cursor position
    pub cursor: usize,
}

impl InputComponent {
    /// Create a new input component
    pub fn new(theme: Theme) -> Self {
        Self {
            theme,
            value: String::new(),
            placeholder: "Type a message...",
            focused: false,
            cursor: 0,
        }
    }

    /// Set the input value
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor = self.value.len();
    }

    /// Set the placeholder text
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.placeholder = placeholder.into();
    }

    /// Check if the input is focused
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Set focus state
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Handle a character input
    pub fn handle_char(&mut self, c: char) {
        if c == '\u{7f}' { // Backspace
            if self.cursor > 0 {
                self.value.remove(self.cursor - 1);
                self.cursor = self.cursor.saturating_sub(1);
            }
        } else if !c.is_control() {
            self.value.insert(self.cursor, c);
            self.cursor = self.cursor.saturating_add(1);
        }
    }

    /// Handle a key input
    pub fn handle_key(&mut self, key: ratatui::event::Key) {
        match key {
            ratatui::event::Key::Char(c) => {
                self.handle_char(c);
            }
            ratatui::event::Key::Delete => {
                if self.cursor < self.value.len() {
                    self.value.remove(self.cursor);
                }
            }
            ratatui::event::Key::Backspace => {
                if self.cursor > 0 {
                    self.value.remove(self.cursor - 1);
                    self.cursor = self.cursor.saturating_sub(1);
                }
            }
            ratatui::event::Key::End => {
                self.cursor = self.value.len();
            }
            ratatui::event::Key::Home => {
                self.cursor = 0;
            }
            ratatui::event::Key::Left => {
                self.cursor = self.cursor.saturating_sub(1);
            }
            ratatui::event::Key::Right => {
                self.cursor = self.cursor.min(self.value.len());
            }
            _ => {}
        }
    }

    /// Clear the input
    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }

    /// Render the input component
    pub fn render(&self, area: Rect, frame: &mut ratatui::Frame) {
        // Determine display text
        let display_text = if self.value.is_empty() {
            self.placeholder.clone()
        } else {
            self.value.clone()
        };

        // Build the text
        let text = Text::styled(display_text, self.theme.foreground);

        // Create the paragraph
        let paragraph = Paragraph::new(text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(if self.focused {
                    self.theme.input_focused()
                } else {
                    self.theme.input()
                }))
            .style(if self.focused {
                self.theme.input_focused()
            } else {
                self.theme.input()
            });

        // Render the paragraph
        frame.render_widget(paragraph, area);

        // Insert a caret if focused
        if self.focused && !self.value.is_empty() {
            // We'll skip the caret for now to keep it simple
            // A more complex implementation would use a custom widget with overlay positioning
        }
    }
}

impl Default for InputComponent {
    fn default() -> Self {
        Self::new(Theme::default())
    }
}