//! UI layer for Model Context Protocol CLI
//!
//! This module provides the Terminal User Interface for interacting with MCP servers
//! using the Ratatui TUI framework.

pub mod app;
pub mod components;
pub mod layout;
pub mod state;
pub mod styles;

pub use app::App;
pub use state::AppMode;