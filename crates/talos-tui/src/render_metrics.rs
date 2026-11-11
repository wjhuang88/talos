//! Opt-in bounded aggregate timings. Never records rendered or input content.
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
pub(crate) enum Stage {
    Draw,
    Layout,
    HistoryRebuild,
    HistoryHit,
    GitScan,
    Diff,
    Flush,
    Input,
    Output,
    Timer,
    InputToSubmit,
    Resize,
    Stream,
    HistoryScroll,
    ScreenResize,
    DeadlineLateness,
    FramePreparation,
    WorkspaceChange,
    GitSnapshotAge,
}
const NAMES: [&str; 19] = [
    "draw",
    "layout",
    "history_rebuild",
    "history_hit",
    "git_scan",
    "diff",
    "flush",
    "input",
    "output",
    "timer",
    "input_to_submit",
    "resize",
    "stream",
    "history_scroll",
    "screen_resize",
    "deadline_lateness",
    "frame_preparation",
    "workspace_change",
    "git_snapshot_age",
];
#[derive(Clone, Copy, Default)]
struct Metric {
    count: u64,
    total_ns: u64,
    max_ns: u64,
    buckets: [u64; 32],
}
struct Metrics {
    steady_stages: [Metric; NAMES.len()],
    started_at: Option<Instant>,
    warmup: Duration,
    steady_changed_cells: u64,
    stages: [Metric; NAMES.len()],
    changed_cells: u64,
    last_screen_size: Option<(u16, u16)>,
}
const EMPTY: Metric = Metric {
    count: 0,
    total_ns: 0,
    max_ns: 0,
    buckets: [0; 32],
};
static METRICS: Mutex<Metrics> = Mutex::new(Metrics {
    steady_stages: [EMPTY; NAMES.len()],
    started_at: None,
    warmup: Duration::ZERO,
    steady_changed_cells: 0,
    stages: [Metric {
        count: 0,
        total_ns: 0,
        max_ns: 0,
        buckets: [0; 32],
    }; NAMES.len()],
    changed_cells: 0,
    last_screen_size: None,
});
static DESTINATION: OnceLock<Option<std::path::PathBuf>> = OnceLock::new();
fn destination() -> &'static Option<std::path::PathBuf> {
    DESTINATION.get_or_init(|| {
        std::env::var_os("TALOS_TUI_METRICS")
            .filter(|v| !v.is_empty())
            .map(Into::into)
    })
}
pub(crate) struct Timer(Option<(Stage, Instant)>);
pub(crate) fn measure(stage: Stage) -> Timer {
    Timer(destination().as_ref().map(|_| (stage, Instant::now())))
}
impl Drop for Timer {
    fn drop(&mut self) {
        if let Some((stage, start)) = self.0 {
            record(
                stage,
                start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64,
            );
        }
    }
}
fn record(stage: Stage, ns: u64) {
    if let Ok(mut metrics) = METRICS.lock() {
        metrics.observe(stage, ns);
    }
}
pub(crate) fn count(stage: Stage) {
    if destination().is_some() {
        record(stage, 0);
    }
}
pub(crate) fn cells(count: usize) {
    if destination().is_some()
        && let Ok(mut metrics) = METRICS.lock()
    {
        metrics.changed_cells = metrics.changed_cells.saturating_add(count as u64);
        if metrics.in_window(0) {
            metrics.steady_changed_cells =
                metrics.steady_changed_cells.saturating_add(count as u64);
        }
    }
}
/// Write once when the UI loop exits, including an error return. Diagnostics must
/// not turn a successfully restored terminal into a failed business operation.
pub(crate) struct ReportOnExit;
impl ReportOnExit {
    pub(crate) fn new() -> Self {
        if destination().is_some()
            && let Ok(mut metrics) = METRICS.lock()
        {
            metrics.steady_stages = [EMPTY; NAMES.len()];
            metrics.steady_changed_cells = 0;
            metrics.last_screen_size = None;
            metrics.started_at = Some(Instant::now());
            metrics.warmup = Duration::from_millis(
                std::env::var("TALOS_TUI_METRICS_WARMUP_MS")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .filter(|v| *v <= 3_600_000)
                    .unwrap_or(0),
            );
        }
        Self
    }
}
impl Metrics {
    fn in_window(&self, duration_ns: u64) -> bool {
        self.started_at.is_some_and(|start| {
            start
                .elapsed()
                .saturating_sub(Duration::from_nanos(duration_ns))
                >= self.warmup
        })
    }
    fn observe(&mut self, stage: Stage, ns: u64) {
        self.stages[stage as usize].observe(ns);
        if self.in_window(ns) {
            self.steady_stages[stage as usize].observe(ns);
        }
    }
}
fn stage_json(values: &[Metric; NAMES.len()]) -> serde_json::Map<String, serde_json::Value> {
    NAMES
        .iter()
        .zip(values)
        .map(|(name, metric)| {
            (
                (*name).into(),
                serde_json::json!({
                    "count": metric.count, "total_ns": metric.total_ns,
                    "max_ns": metric.max_ns, "log2_ns_buckets": metric.buckets,
                }),
            )
        })
        .collect()
}
impl Drop for ReportOnExit {
    fn drop(&mut self) {
        let Some(path) = destination() else {
            return;
        };
        let Ok(metrics) = METRICS.lock() else {
            return;
        };
        let elapsed = metrics
            .started_at
            .map(|start| start.elapsed())
            .unwrap_or_default();
        let report = serde_json::json!({"schema": 2, "stages": stage_json(&metrics.stages),
            "changed_cells": metrics.changed_cells,
            "steady_stages": stage_json(&metrics.steady_stages),
            "steady_changed_cells": metrics.steady_changed_cells,
            "warmup_ms": metrics.warmup.as_millis(),
            "steady_elapsed_ns": elapsed.saturating_sub(metrics.warmup).as_nanos(),
            "window_origin": "Tui::run entry; excludes durations crossing warmup boundary",
            "scope": "stages are process cumulative; steady_stages cover current UI run after warmup; nested durations are not additive"});
        if let Ok(bytes) = serde_json::to_vec_pretty(&report) {
            let _ = std::fs::write(path, bytes);
        }
    }
}

/// Software latency ends at a successful frame submission, not visible pixels.
pub(crate) fn start() -> Option<Instant> {
    destination().as_ref().map(|_| Instant::now())
}
pub(crate) fn submitted(start: Instant) {
    record(
        Stage::InputToSubmit,
        start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64,
    );
}

impl Metric {
    fn observe(&mut self, ns: u64) {
        self.count = self.count.saturating_add(1);
        self.total_ns = self.total_ns.saturating_add(ns);
        self.max_ns = self.max_ns.max(ns);
        let bucket = (64 - ns.leading_zeros()).min(31) as usize;
        self.buckets[bucket] = self.buckets[bucket].saturating_add(1);
    }
}

/// Count actual terminal-size changes observed by rendering, not injected events.
pub(crate) fn screen_size(width: u16, height: u16) {
    if destination().is_none() {
        return;
    }
    if let Ok(mut metrics) = METRICS.lock() {
        let size = (width, height);
        if metrics
            .last_screen_size
            .is_some_and(|previous| previous != size)
        {
            metrics.observe(Stage::ScreenResize, 0);
        }
        metrics.last_screen_size = Some(size);
    }
}

/// Age of a successful snapshot when read for a frame, not a scan duration.
/// Sampling is frame-weighted; unchanged idle snapshots do not generate samples.
pub(crate) fn snapshot_age(age: Duration) {
    if destination().is_some()
        && let Ok(mut metrics) = METRICS.lock()
    {
        let ns = age.as_nanos().min(u128::from(u64::MAX)) as u64;
        metrics.stages[Stage::GitSnapshotAge as usize].observe(ns);
        // The age is a gauge observed now, so it does not cross the warmup boundary.
        if metrics.in_window(0) {
            metrics.steady_stages[Stage::GitSnapshotAge as usize].observe(ns);
        }
    }
}

pub(crate) fn deadline(scheduled: tokio::time::Instant) {
    if destination().is_some() {
        record(
            Stage::DeadlineLateness,
            scheduled.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::Metric;

    #[test]
    fn histogram_bounds_include_zero_and_overflow_without_wrapping() {
        let mut metric = Metric::default();
        for ns in [0, 1, 2, 3, 1 << 31, u64::MAX] {
            metric.observe(ns);
        }
        assert_eq!(metric.count, 6);
        assert_eq!(metric.buckets.iter().sum::<u64>(), 6);
        assert_eq!(metric.buckets[0], 1);
        assert_eq!(metric.buckets[2], 2);
        assert_eq!(metric.buckets[31], 2);
        assert_eq!(metric.total_ns, u64::MAX);
        metric.count = u64::MAX;
        metric.buckets[0] = u64::MAX;
        metric.observe(0);
        assert_eq!(metric.count, u64::MAX);
        assert_eq!(metric.buckets[0], u64::MAX);
    }
}
