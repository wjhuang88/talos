//! Final, bounded component rectangles for one full-frame render.

use ratatui::layout::{Rect, Size};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ComponentMetrics {
    pub(crate) preview: u16,
    pub(crate) queue: u16,
    pub(crate) tips: u16,
    pub(crate) panel: u16,
    pub(crate) composer: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AppLayout {
    pub(crate) root: Rect,
    pub(crate) history: Option<Rect>,
    pub(crate) preview: Option<Rect>,
    pub(crate) queue: Option<Rect>,
    pub(crate) tips: Option<Rect>,
    pub(crate) panel: Option<Rect>,
    pub(crate) composer_top_pad: Option<Rect>,
    pub(crate) composer: Option<Rect>,
    pub(crate) composer_bottom_pad: Option<Rect>,
    pub(crate) status: Option<Rect>,
}

pub(crate) fn compute_app_layout(size: Size, metrics: ComponentMetrics) -> AppLayout {
    let root = Rect::new(0, 0, size.width, size.height);
    let top = 0;
    let mut bottom = size.height;
    let composer_height = if bottom.saturating_sub(top) >= 2 {
        metrics.composer.max(1).min(bottom.saturating_sub(top) - 1)
    } else {
        bottom.saturating_sub(top)
    };
    let mut take_bottom = |wanted: u16| {
        let height = wanted.min(bottom.saturating_sub(top));
        (height > 0).then(|| {
            bottom = bottom.saturating_sub(height);
            Rect::new(0, bottom, size.width, height)
        })
    };

    // Hard priority: composer, then status, then active/optional panes.
    let composer = take_bottom(composer_height);
    let status = take_bottom(1);
    let composer_bottom_pad = take_bottom(1);
    let composer_top_pad = take_bottom(1);
    let panel = take_bottom(metrics.panel);
    let tips = take_bottom(metrics.tips);
    let queue = take_bottom(metrics.queue);
    let preview = take_bottom(metrics.preview);
    let history = (bottom > top).then(|| Rect::new(0, top, size.width, bottom - top));

    AppLayout {
        root,
        history,
        preview,
        queue,
        tips,
        panel,
        composer_top_pad,
        composer,
        composer_bottom_pad,
        status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn app_layout_assigns_component_level_rects_with_composer_priority() {
        for (width, height) in [(0, 0), (1, 1), (1, 2), (2, 2), (3, 3), (40, 5), (80, 24)] {
            let layout = compute_app_layout(
                Size::new(width, height),
                ComponentMetrics {
                    preview: 3,
                    queue: 2,
                    tips: 1,
                    panel: 4,
                    composer: 3,
                },
            );
            let rects = [
                layout.history,
                layout.preview,
                layout.queue,
                layout.tips,
                layout.panel,
                layout.composer_top_pad,
                layout.composer,
                layout.composer_bottom_pad,
                layout.status,
            ];
            for rect in rects.into_iter().flatten() {
                assert!(
                    rect.right() <= layout.root.right() && rect.bottom() <= layout.root.bottom()
                );
            }
            if height == 1 {
                assert_eq!(layout.composer.map(|r| r.height), Some(1));
                assert!(layout.status.is_none());
            }
            if height >= 2 {
                assert!(layout.composer.is_some() && layout.status.is_some());
            }
        }
    }
}
