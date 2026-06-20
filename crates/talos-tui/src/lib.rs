mod app;
mod clipboard;
pub mod evolution;
mod export;
pub mod formatting;
mod highlight;
mod inline_terminal;
mod scrollback;
mod sidebar;
mod splash;
mod state;
mod stream_markdown;
mod theme;
mod tool_display;
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
