//! Bounded allocation for the full-frame renderer.

use ratatui::layout::{Rect, Size};

/// The bottom pane is allocated from the terminal bottom; history receives the
/// remaining rows. The caller may subdivide `bottom`, but must remain inside it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AppLayout {
    pub(crate) root: Rect,
    pub(crate) history: Rect,
    pub(crate) bottom: Rect,
}

pub(crate) fn compute_app_layout(size: Size, requested_bottom_height: u16) -> AppLayout {
    let root = Rect::new(0, 0, size.width, size.height);
    let bottom_height = requested_bottom_height.min(size.height);
    let history_height = size.height.saturating_sub(bottom_height);
    AppLayout {
        root,
        history: Rect::new(0, 0, size.width, history_height),
        bottom: Rect::new(0, history_height, size.width, bottom_height),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_layout_rects_are_bounded_and_non_overlapping() {
        for (width, height) in [
            (0, 0),
            (1, 1),
            (1, 2),
            (2, 1),
            (2, 2),
            (3, 3),
            (5, 2),
            (20, 3),
            (40, 5),
            (80, 24),
        ] {
            let layout = compute_app_layout(Size::new(width, height), 10);
            for rect in [layout.history, layout.bottom] {
                assert!(rect.right() <= layout.root.right());
                assert!(rect.bottom() <= layout.root.bottom());
            }
            assert!(layout.history.bottom() <= layout.bottom.y);
        }
    }
}
