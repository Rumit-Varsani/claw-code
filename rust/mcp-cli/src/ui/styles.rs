//! TUI styles and color combinations

use ratatui::style::{Color, Style};

/// Theme configuration for the application
#[derive(Debug, Clone)]
pub struct Theme {
    /// Background color
    pub background: Color,

    /// Foreground color
    pub foreground: Color,

    /// Primary accent color
    pub primary: Color,

    /// Secondary accent color
    pub secondary: Color,

    /// Success color
    pub success: Color,

    /// Error color
    pub error: Color,

    /// Warning color
    pub warning: Color,

    /// Info color
    pub info: Color,

    /// Inactive color (dimmed)
    pub inactive: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::Rgb(30, 30, 35),
            foreground: Color::Rgb(230, 230, 235),
            primary: Color::Rgb(86, 156, 214),
            secondary: Color::Rgb(170, 191, 252),
            success: Color::Rgb(88, 166, 114),
            error: Color::Rgb(235, 107, 83),
            warning: Color::Rgb(234, 177, 75),
            info: Color::Rgb(129, 161, 193),
            inactive: Color::Rgb(100, 110, 120),
        }
    }
}

impl Theme {
    /// Create a dark theme
    pub fn dark() -> Self {
        Self {
            background: Color::Rgb(27, 31, 36),
            foreground: Color::Rgb(220, 223, 228),
            primary: Color::Rgb(56, 139, 253),
            secondary: Color::Rgb(103, 148, 255),
            success: Color::Rgb(46, 160, 87),
            error: Color::Rgb(235, 84, 53),
            warning: Color::Rgb(208, 137, 60),
            info: Color::Rgb(91, 155, 213),
            inactive: Color::Rgb(155, 155, 155),
        }
    }

    /// Create a light theme
    pub fn light() -> Self {
        Self {
            background: Color::Rgb(249, 250, 251),
            foreground: Color::Rgb(30, 30, 36),
            primary: Color::Rgb(29, 100, 211),
            secondary: Color::Rgb(66, 133, 244),
            success: Color::Rgb(52, 168, 83),
            error: Color::Rgb(227, 58, 47),
            warning: Color::Rgb(239, 112, 0),
            info: Color::Rgb(97, 179, 239),
            inactive: Color::Rgb(139, 139, 139),
        }
    }

    /// Apply theme to a style
    pub fn style(&self) -> Style {
        Style::default().bg(self.background).fg(self.foreground)
    }

    /// Normal text style
    pub fn normal_text(&self) -> Style {
        Style::default().fg(self.foreground)
    }

    /// Title style
    pub fn title(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(ratatui::style::Modifier::BOLD)
    }

    /// Primary widget style
    pub fn primary(&self) -> Style {
        Style::default().fg(self.primary)
    }

    /// Secondary widget style
    pub fn secondary(&self) -> Style {
        Style::default().fg(self.secondary)
    }

    /// Success style
    pub fn success(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// Error style
    pub fn error(&self) -> Style {
        Style::default().fg(self.error)
    }

    /// Warning style
    pub fn warning(&self) -> Style {
        Style::default().fg(self.warning)
    }

    /// Info style
    pub fn info(&self) -> Style {
        Style::default().fg(self.info)
    }

    /// Inactive style
    pub fn inactive(&self) -> Style {
        Style::default()
            .fg(self.inactive)
            .add_modifier(ratatui::style::Modifier::DIM)
    }

    /// Block style
    pub fn block(&self) -> Style {
        Style::default()
            .fg(self.background)
            .add_modifier(ratatui::style::Modifier::BOLD)
    }

    /// Input field style
    pub fn input(&self) -> Style {
        Style::default()
            .bg(self.background)
            .fg(self.foreground)
            .add_modifier(ratatui::style::Modifier::UNDERLINED)
    }

    /// Focused input style
    pub fn input_focused(&self) -> Style {
        Style::default()
            .bg(self.primary)
            .fg(self.background)
            .add_modifier(ratatui::style::Modifier::BOLD)
    }
}

/// Helper macro for creating styles
macro_rules! theme {
    ($bg:expr, $fg:expr, $primary:expr, $secondary:expr, $success:expr, $error:expr, $warning:expr, $info:expr, $inactive:expr) => {
        Theme {
            background: Color::Rgb($bg.0, $bg.1, $bg.2),
            foreground: Color::Rgb($fg.0, $fg.1, $fg.2),
            primary: Color::Rgb($primary.0, $primary.1, $primary.2),
            secondary: Color::Rgb($secondary.0, $secondary.1, $secondary.2),
            success: Color::Rgb($success.0, $success.1, $success.2),
            error: Color::Rgb($error.0, $error.1, $error.2),
            warning: Color::Rgb($warning.0, $warning.1, $warning.2),
            info: Color::Rgb($info.0, $info.1, $info.2),
            inactive: Color::Rgb($inactive.0, $inactive.1, $inactive.2),
        }
    };
}

pub use theme;