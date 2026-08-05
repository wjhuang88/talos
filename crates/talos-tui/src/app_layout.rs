//! Final, bounded component rectangles for one full-frame render.

use ratatui::layout::{Rect, Size};

use crate::scrollback::BottomPanelPlacement;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ComponentMetrics {
    pub(crate) preview: u16,
    pub(crate) queue: u16,
    pub(crate) tips: u16,
    pub(crate) panel_required: u16,
    pub(crate) panel_preferred: u16,
    pub(crate) composer: u16,
    /// When set, history height is capped at this value instead of consuming
    /// all remaining space. Used by the startup inline-composer layout so the
    /// composer sits just below the Logo virtual prefix rather than at the
    /// terminal bottom.
    pub(crate) history_cap: Option<u16>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct AllocatedComponentHeights {
    history: u16,
    preview: u16,
    queue: u16,
    tips: u16,
    panel: u16,
    composer_top_pad: u16,
    composer: u16,
    composer_bottom_pad: u16,
    status: u16,
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

pub(crate) fn compute_app_layout(
    size: Size,
    metrics: ComponentMetrics,
    placement: BottomPanelPlacement,
) -> AppLayout {
    let heights = allocate_heights(size.height, metrics);
    place_rects(size, heights, placement)
}

fn allocate_heights(total: u16, metrics: ComponentMetrics) -> AllocatedComponentHeights {
    let mut remaining = total;
    let mut take = |wanted: u16| {
        let allocated = wanted.min(remaining);
        remaining = remaining.saturating_sub(allocated);
        allocated
    };

    let composer = take(u16::from(metrics.composer > 0));
    let status = take(1);
    let panel_required = take(metrics.panel_required.min(metrics.panel_preferred));
    let composer_top_pad = take(u16::from(metrics.composer > 0));
    let composer_bottom_pad = take(u16::from(metrics.composer > 0));
    let tips = take(metrics.tips);
    let preview = take(metrics.preview);
    let queue = take(metrics.queue);
    let panel_optional = take(metrics.panel_preferred.saturating_sub(panel_required));
    let composer_extra = take(metrics.composer.saturating_sub(composer));
    let history = match metrics.history_cap {
        Some(cap) => remaining.min(cap),
        None => remaining,
    };

    AllocatedComponentHeights {
        history,
        preview,
        queue,
        tips,
        panel: panel_required.saturating_add(panel_optional),
        composer_top_pad,
        composer: composer.saturating_add(composer_extra),
        composer_bottom_pad,
        status,
    }
}

fn place_rects(
    size: Size,
    heights: AllocatedComponentHeights,
    placement: BottomPanelPlacement,
) -> AppLayout {
    let root = Rect::new(0, 0, size.width, size.height);
    let mut y = 0;
    let mut take = |height: u16| {
        (height > 0).then(|| {
            let rect = Rect::new(0, y, size.width, height);
            y = y.saturating_add(height);
            rect
        })
    };

    let history = take(heights.history);
    let preview = take(heights.preview);
    let queue = take(heights.queue);
    let tips = take(heights.tips);
    let (panel, composer_top_pad, composer) = match placement {
        BottomPanelPlacement::AboveInput => (
            take(heights.panel),
            take(heights.composer_top_pad),
            take(heights.composer),
        ),
        BottomPanelPlacement::BelowInput => {
            let top_pad = take(heights.composer_top_pad);
            let composer = take(heights.composer);
            (take(heights.panel), top_pad, composer)
        }
    };
    let composer_bottom_pad = take(heights.composer_bottom_pad);
    let status = take(heights.status);

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

    fn metrics() -> ComponentMetrics {
        ComponentMetrics {
            preview: 1,
            queue: 2,
            tips: 1,
            panel_required: 2,
            panel_preferred: 4,
            composer: 3,
            history_cap: None,
        }
    }

    fn assert_bounded_and_ordered(layout: AppLayout) {
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
        let mut visible = rects.into_iter().flatten().collect::<Vec<_>>();
        visible.sort_by_key(|rect| rect.y);
        for rect in &visible {
            assert!(rect.right() <= layout.root.right());
            assert!(rect.bottom() <= layout.root.bottom());
        }
        for pair in visible.windows(2) {
            assert!(pair[0].bottom() <= pair[1].y);
        }
    }

    #[test]
    fn visual_order_above_input_is_correct() {
        let layout = compute_app_layout(
            Size::new(80, 24),
            metrics(),
            BottomPanelPlacement::AboveInput,
        );
        assert_eq!(
            layout.panel.expect("operation should succeed").bottom(),
            layout.composer_top_pad.expect("operation should succeed").y
        );
        assert_eq!(
            layout
                .composer_top_pad
                .expect("operation should succeed")
                .bottom(),
            layout.composer.expect("operation should succeed").y
        );
        assert_eq!(
            layout.composer.expect("operation should succeed").bottom(),
            layout
                .composer_bottom_pad
                .expect("operation should succeed")
                .y
        );
        assert_eq!(
            layout
                .composer_bottom_pad
                .expect("operation should succeed")
                .bottom(),
            layout.status.expect("operation should succeed").y
        );
        assert_bounded_and_ordered(layout);
    }

    #[test]
    fn visual_order_below_input_is_correct() {
        let layout = compute_app_layout(
            Size::new(80, 24),
            metrics(),
            BottomPanelPlacement::BelowInput,
        );
        assert_eq!(
            layout
                .composer_top_pad
                .expect("operation should succeed")
                .bottom(),
            layout.composer.expect("operation should succeed").y
        );
        assert_eq!(
            layout.composer.expect("operation should succeed").bottom(),
            layout.panel.expect("operation should succeed").y
        );
        assert_eq!(
            layout.panel.expect("operation should succeed").bottom(),
            layout
                .composer_bottom_pad
                .expect("operation should succeed")
                .y
        );
        assert_eq!(
            layout
                .composer_bottom_pad
                .expect("operation should succeed")
                .bottom(),
            layout.status.expect("operation should succeed").y
        );
        assert_bounded_and_ordered(layout);
    }

    #[test]
    fn app_layout_extreme_sizes_preserve_required_priority() {
        for (width, height) in [
            (0, 0),
            (1, 1),
            (1, 2),
            (2, 1),
            (2, 2),
            (3, 2),
            (3, 3),
            (5, 2),
            (10, 3),
            (20, 5),
            (80, 24),
        ] {
            let layout = compute_app_layout(
                Size::new(width, height),
                metrics(),
                BottomPanelPlacement::AboveInput,
            );
            assert_bounded_and_ordered(layout);
            if height == 1 {
                assert_eq!(layout.composer.map(|rect| rect.height), Some(1));
                assert!(layout.status.is_none());
            }
            if height >= 2 {
                assert!(layout.composer.is_some());
                assert!(layout.status.is_some());
            }
        }
    }
}
