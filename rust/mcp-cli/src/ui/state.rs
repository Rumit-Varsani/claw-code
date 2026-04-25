//! Application state and modes for the TUI

/// Application modes representing different UI states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    /// Normal chat mode
    Chat,
    /// Command palette (useful for searching commands)
    CommandPalette,
    /// View message history
    History,
    /// Toggle help screen
    Help,
    /// Settings configuration
    Settings,
    /// System status
    Status,
}

impl AppMode {
    /// Check if this is a view-only mode
    pub fn is_view_only(&self) -> bool {
        matches!(self, Self::History | Self::Help | Self::Settings | Self::Status)
    }

    /// Check if navigation keys should be enabled
    pub fn navigable(&self) -> bool {
        matches!(
            self,
            Self::Chat | Self::History | Self::CommandPalette
        )
    }

    /// Get the display name for this mode
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Chat => "Chat",
            Self::CommandPalette => "Command",
            Self::History => "History",
            Self::Help => "Help",
            Self::Settings => "Settings",
            Self::Status => "Status",
        }
    }
}

/// Status indicator types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusIndicator {
    /// Normal operation
    Normal,
    /// Processing request
    Loading,
    /// Error occurred
    Error,
    /// Success operation
    Success,
    /// Warning/disabled state
    Warning,
}

impl StatusIndicator {
    pub fn color(&self) -> &'static str {
        match self {
            Self::Normal => "green",
            Self::Loading => "yellow",
            Self::Error => "red",
            Self::Success => "green",
            Self::Warning => "yellow",
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Normal => "●",
            Self::Loading => "⟳",
            Self::Error => "✗",
            Self::Success => "✓",
            Self::Warning => "!"
        }
    }
}

/// Cursor state for navigation
#[derive(Debug, Clone, Copy, Default)]
pub enum CursorPosition {
    #[default]
    Bottom,
    Middle,
    Top,
}

/// Input focus states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusState {
    /// No specific focus
    None,
    /// Text input area
    Input,
    /// Command palette input
    CommandInput,
    /// Scrollable content
    Scrollable,
}

/// Message visibility filter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageFilter {
    /// Show all messages
    All,
    /// Show only user messages
    UserOnly,
    /// Show only assistant responses
    AssistantOnly,
    /// Show only error messages
    Errors,
}