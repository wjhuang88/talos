//! TUI presentation for the shared text classifier.

#[cfg(test)]
pub(crate) use talos_text::stream::BoundaryHint;

pub(crate) use talos_text::stream::{
    BlockDecision, FallbackReason, HoldStatus, MarkdownBlockKind, StreamBlockClassifier,
};

pub(crate) fn preview_text(status: &HoldStatus) -> &'static str {
    match status.kind {
        MarkdownBlockKind::CodeFence => "receiving code block...",
        MarkdownBlockKind::Table => "rendering table...",
        MarkdownBlockKind::List => "formatting list...",
        MarkdownBlockKind::Quote => "formatting quote...",
    }
}
