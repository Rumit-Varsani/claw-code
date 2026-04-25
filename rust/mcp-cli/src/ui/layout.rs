//! Layout definitions for the TUI application

use ratatui::layout::{Alignment, Constraint, Direction, Rect, Splitter};

use super::state::AppMode;

/// Layout configurations for different application modes
#[derive(Debug, Clone, Default)]
pub struct AppLayout {
    /// Current application mode
    pub mode: AppMode,

    /// Main content area layout
    pub main: LayoutRegion,

    /// Sidebar panel layout
    pub sidebar: Option<LayoutRegion>,

    /// Status bar layout
    pub status: LayoutRegion,

    /// Input area layout
    pub input: LayoutRegion,
}

#[derive(Debug, Clone, Default)]
pub struct LayoutRegion {
    /// Constraints for this region
    pub constraints: Vec<Constraint>,

    /// Padding for the region
    pub padding: (u16, u16, u16, u16),

    /// Whether this region should be scrollable
    pub scrollable: bool,
}

impl AppLayout {
    /// Create a new layout for the given mode
    pub fn for_mode(mode: AppMode) -> Self {
        match mode {
            AppMode::Chat => Self::chat_layout(),
            AppMode::History => Self::history_layout(),
            AppMode::Help => Self::help_layout(),
            AppMode::Settings => Self::settings_layout(),
            AppMode::Status => Self::status_layout(),
            AppMode::CommandPalette => Self::command_palette_layout(),
        }
    }

    /// Default chat layout
    fn chat_layout() -> Self {
        Self {
            mode: AppMode::Chat,
            main: LayoutRegion {
                constraints: vec![
                    Constraint::Min(1),  // Sidebar (optional)
                    Constraint::Percentage(60),
                ],
                padding: (0, 0, 0, 0),
                scrollable: true,
            },
            sidebar: None,
            status: LayoutRegion {
                constraints: vec![
                    Constraint::Length(1),
                ],
                padding: (0, 0, 0, 0),
                scrollable: false,
            },
            input: LayoutRegion {
                constraints: vec![
                    Constraint::Length(4),
                ],
                padding: (1, 0, 1, 0),
                scrollable: false,
            },
        }
    }

    /// History view layout
    fn history_layout() -> Self {
        Self {
            mode: AppMode::History,
            main: LayoutRegion {
                constraints: vec![
                    Constraint::Percentage(100),
                ],
                padding: (0, 0, 1, 0),
                scrollable: true,
            },
            sidebar: Some(LayoutRegion {
                constraints: vec![
                    Constraint::Percentage(20),
                ],
                padding: (0, 0, 0, 0),
                scrollable: true,
            }),
            status: LayoutRegion {
                constraints: vec![
                    Constraint::Length(1),
                ],
                padding: (0, 0, 0, 0),
                scrollable: false,
            },
            input: LayoutRegion {
                constraints: vec![
                    Constraint::Length(2),
                ],
                padding: (1, 0, 1, 0),
                scrollable: false,
            },
        }
    }

    /// Help view layout
    fn help_layout() -> Self {
        Self {
            mode: AppMode::Help,
            main: LayoutRegion {
                constraints: vec![
                    Constraint::Percentage(100),
                ],
                padding: (1, 1, 1, 1),
                scrollable: true,
            },
            sidebar: None,
            status: LayoutRegion {
                constraints: vec![
                    Constraint::Length(1),
                ],
                padding: (0, 0, 0, 0),
                scrollable: false,
            },
            input: LayoutRegion {
                constraints: vec![
                    Constraint::Length(0),  // No input in help
                ],
                padding: (0, 0, 0, 0),
                scrollable: false,
            },
        }
    }

    /// Settings view layout
    fn settings_layout() -> Self {
        Self {
            mode: AppMode::Settings,
            main: LayoutRegion {
                constraints: vec![
                    Constraint::Percentage(100),
                ],
                padding: (0, 0, 0, 0),
                scrollable: false,
            },
            sidebar: Some(LayoutRegion {
                constraints: vec![
                    Constraint::Percentage(15),
                ],
                padding: (0, 0, 0, 0),
                scrollable: false,
            }),
            status: LayoutRegion {
                constraints: vec![
                    Constraint::Length(1),
                ],
                padding: (0, 0, 0, 0),
                scrollable: false,
            },
            input: LayoutRegion {
                constraints: vec![
                    Constraint::Length(0),  // No input in settings
                ],
                padding: (0, 0, 0, 0),
                scrollable: false,
            },
        }
    }

    /// Status view layout
    fn status_layout() -> Self {
        Self {
            mode: AppMode::Status,
            main: LayoutRegion {
                constraints: vec![
                    Constraint::Percentage(80),
                ],
                padding: (1, 0, 1, 0),
                scrollable: true,
            },
            sidebar: Some(LayoutRegion {
                constraints: vec![
                    Constraint::Percentage(20),
                ],
                padding: (0, 0, 0, 0),
                scrollable: false,
            }),
            status: LayoutRegion {
                constraints: vec![
                    Constraint::Length(4),
                ],
                padding: (0, 0, 0, 0),
                scrollable: false,
            },
            input: LayoutRegion {
                constraints: vec![
                    Constraint::Length(0),
                ],
                padding: (0, 0, 0, 0),
                scrollable: false,
            },
        }
    }

    /// Command palette layout
    fn command_palette_layout() -> Self {
        Self {
            mode: AppMode::CommandPalette,
            main: LayoutRegion {
                constraints: vec![
                    Constraint::Percentage(100),
                ],
                padding: (0, 0, 0, 0),
                scrollable: false,
            },
            sidebar: None,
            status: LayoutRegion {
                constraints: vec![
                    Constraint::Length(0),
                ],
                padding: (0, 0, 0, 0),
                scrollable: false,
            },
            input: LayoutRegion {
                constraints: vec![
                    Constraint::Length(1),  // For transparent input if needed
                ],
                padding: (0, 0, 0, 0),
                scrollable: false,
            },
        }
    }

    /// Apply layout constraints to a given frame
    pub fn apply(&self, frame: &mut ratatui::Frame) {
        self.apply_region(frame, self.main, 0)
    }

    /// Apply a nested layout region
    fn apply_region(
        &self,
        frame: &mut ratatui::Frame,
        region: &LayoutRegion,
        offset: RatatOffset,
    ) -> Rect {
        let area = frame.area();
        let rect = match offset {
            RatatOffset::None => area,
            RatatOffset::Sidebar => {
                if let Some(sidebar) = self.sidebar.clone() {
                    self.apply_region(frame, &sidebar, RatatOffset::None)
                } else {
                    area
                }
            }
            RatatOffset::Main => area,
        };

        let inner = self.inner_rect(&rect, region.padding);

        // Apply constraints using splitter
        let constraints = self.constraints_for_region(region);
        let splitter = Splitter::new()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(rect);

        if splitter.is_empty() {
            inner
        } else {
            splitter[0]
        }
    }

    /// Get constraints from a layout region
    fn constraints_for_region(&self, region: &LayoutRegion) -> Vec<Constraint> {
        region.constraints.clone()
    }

    /// Create inner rect with padding
    fn inner_rect(&self, outer: Rect, padding: (u16, u16, u16, u16)) -> Rect {
        Rect {
            x: outer.x + padding.0,
            y: outer.y + padding.1,
            width: outer.width.saturating_sub(padding.0 + padding.2),
            height: outer.height.saturating_sub(padding.1 + padding.3),
        }
    }
}

/// Offset type for nested regions
#[derive(Debug, Clone, Copy, Default)]
pub enum RatatOffset {
    #[default]
    None,
    Sidebar,
    Main,
}

/// Create a centered horizontal split layout
pub fn centered_split<'a>(
    area: Rect,
    constraints: &'a [Constraint],
) -> Vec<Rect> {
    let splitter = Splitter::new()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);
    splitter.to_vec()
}

/// Create a centered vertical split layout
pub fn centered_vertical_split<'a>(
    area: Rect,
    constraints: &'a [Constraint],
) -> Vec<Rect> {
    let splitter = Splitter::new()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);
    splitter.to_vec()
}

/// Create a grid layout for panels
pub fn grid_layout(area: Rect, rows: u16, cols: u16) -> Vec<Rect> {
    let mut positions = vec![];
    let cell_width = area.width / cols;
    let cell_height = area.height / rows;

    for y in 0..rows {
        for x in 0..cols {
            positions.push(Rect {
                x: area.x + (x * cell_width),
                y: area.y + (y * cell_height),
                width: cell_width,
                height: cell_height,
            });
        }
    }

    positions
}