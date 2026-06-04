//! Talos TUI - terminal user interface for the Talos agent.
//!
//! Provides a chat-based interface with streaming output, tool call rendering,
//! permission approval overlays, slash commands, and status panels.

mod app;
pub mod evolution;
mod sidebar;
mod state;
mod theme;
mod widgets;

pub use app::Tui;
pub use sidebar::{SkillInfo, SkillSidebar};
pub use state::ApprovalState;
pub use theme::nord;
pub use widgets::{ApprovalOverlay, ToolCallBubble};

#[cfg(test)]
pub(crate) use theme::{contrast_ratio, rgb_components};
#[cfg(test)]
mod tests;
