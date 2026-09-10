//! Renderer-neutral Markdown line classification; callers own chunk assembly and rendering.

const MAX_HELD_LINES: usize = 200;
const MAX_HELD_BYTES: usize = 16 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
/// Semantic kind of a held Markdown block.
pub enum MarkdownBlockKind {
    /// Backtick or tilde fenced code.
    CodeFence,
    /// Pipe-delimited table.
    Table,
    /// Ordered or unordered list.
    List,
    /// Block quote.
    Quote,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Boundary expected before emitting a held block.
pub enum BoundaryHint {
    /// Matching closing fence.
    CodeFenceClose,
    /// Separator confirming a candidate table.
    TableSeparator,
    /// First non-table line.
    TableEnd,
    /// First non-list line.
    NonListLine,
    /// First non-quote line.
    NonQuoteLine,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Cause of plain-source fallback.
pub enum FallbackReason {
    /// Held source budget exceeded.
    HeldBlockTooLarge,
    /// End of input before closing fence.
    UnterminatedCodeFence,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Renderer-neutral progress of a held block.
pub struct HoldStatus {
    /// Semantic block kind.
    pub kind: MarkdownBlockKind,
    /// Buffered source lines or their count.
    pub lines: usize,
    /// Source bytes excluding newline separators.
    pub bytes: usize,
    /// Expected boundary.
    pub boundary_hint: BoundaryHint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// Classification decision containing original source lines.
pub enum BlockDecision {
    /// Emit an ordinary source line.
    ImmediateLine(String),
    /// Begin buffering a semantic block.
    StartHold {
        /// Block progress.
        status: HoldStatus,
    },
    /// Continue buffering a semantic block.
    ContinueHold {
        /// Block progress.
        status: HoldStatus,
    },
    /// Emit source for a complete semantic block.
    FinishHold {
        /// Block progress.
        status: HoldStatus,
        /// Semantic block kind.
        kind: MarkdownBlockKind,
        /// Buffered source lines or their count.
        lines: Vec<String>,
    },
    /// Emit source without relying on complete block structure.
    FallbackImmediate {
        /// Block progress.
        status: HoldStatus,
        /// Semantic block kind.
        kind: MarkdownBlockKind,
        /// Reason for fallback.
        reason: FallbackReason,
        /// Buffered source lines or their count.
        lines: Vec<String>,
    },
}

#[derive(Default)]
/// Stateful complete-line Markdown classifier.
pub struct StreamBlockClassifier {
    state: ClassifierState,
}

#[derive(Default)]
enum ClassifierState {
    #[default]
    Plain,
    PendingTableHeader(String),
    Holding {
        kind: MarkdownBlockKind,
        lines: Vec<String>,
        fence_marker: Option<(String, usize)>,
    },
}

impl StreamBlockClassifier {
    /// Classify one complete UTF-8 line without its newline terminator.
    pub fn push_line(&mut self, line: String) -> Vec<BlockDecision> {
        let mut decisions = Vec::new();
        let mut next = Some(line);

        while let Some(line) = next.take() {
            match std::mem::take(&mut self.state) {
                ClassifierState::Plain => {
                    self.handle_plain(line, &mut decisions);
                }
                ClassifierState::PendingTableHeader(header) => {
                    if is_table_separator(&line) {
                        let lines = vec![header, line];
                        let status =
                            hold_status(&MarkdownBlockKind::Table, &lines, BoundaryHint::TableEnd);
                        self.state = ClassifierState::Holding {
                            kind: MarkdownBlockKind::Table,
                            lines,
                            fence_marker: None,
                        };
                        decisions.push(BlockDecision::StartHold { status });
                    } else {
                        decisions.push(BlockDecision::ImmediateLine(header));
                        self.state = ClassifierState::Plain;
                        next = Some(line);
                    }
                }
                ClassifierState::Holding {
                    kind,
                    mut lines,
                    fence_marker,
                } => match kind {
                    MarkdownBlockKind::CodeFence => {
                        lines.push(line);
                        let is_closed = lines.last().is_some_and(|last| {
                            fence_marker.as_ref().is_some_and(|(marker, count)| {
                                is_matching_fence_close(last, marker, *count)
                            })
                        });
                        if is_closed {
                            let status = hold_status(
                                &MarkdownBlockKind::CodeFence,
                                &lines,
                                BoundaryHint::CodeFenceClose,
                            );
                            let rendered = render_block(&MarkdownBlockKind::CodeFence, &lines);
                            self.state = ClassifierState::Plain;
                            decisions.push(BlockDecision::FinishHold {
                                status,
                                kind: MarkdownBlockKind::CodeFence,
                                lines: rendered,
                            });
                        } else if held_block_too_large(&lines) {
                            let status = hold_status(
                                &MarkdownBlockKind::CodeFence,
                                &lines,
                                BoundaryHint::CodeFenceClose,
                            );
                            self.state = ClassifierState::Plain;
                            decisions.push(BlockDecision::FallbackImmediate {
                                status,
                                kind: MarkdownBlockKind::CodeFence,
                                reason: FallbackReason::HeldBlockTooLarge,
                                lines,
                            });
                        } else {
                            let status = hold_status(
                                &MarkdownBlockKind::CodeFence,
                                &lines,
                                BoundaryHint::CodeFenceClose,
                            );
                            self.state = ClassifierState::Holding {
                                kind: MarkdownBlockKind::CodeFence,
                                lines,
                                fence_marker,
                            };
                            decisions.push(BlockDecision::ContinueHold { status });
                        }
                    }
                    MarkdownBlockKind::Table => {
                        if is_table_row(&line) {
                            lines.push(line);
                            if held_block_too_large(&lines) {
                                let status = hold_status(
                                    &MarkdownBlockKind::Table,
                                    &lines,
                                    BoundaryHint::TableEnd,
                                );
                                self.state = ClassifierState::Plain;
                                decisions.push(BlockDecision::FallbackImmediate {
                                    status,
                                    kind: MarkdownBlockKind::Table,
                                    reason: FallbackReason::HeldBlockTooLarge,
                                    lines,
                                });
                            } else {
                                let status = hold_status(
                                    &MarkdownBlockKind::Table,
                                    &lines,
                                    BoundaryHint::TableEnd,
                                );
                                self.state = ClassifierState::Holding {
                                    kind: MarkdownBlockKind::Table,
                                    lines,
                                    fence_marker: None,
                                };
                                decisions.push(BlockDecision::ContinueHold { status });
                            }
                        } else {
                            let status = hold_status(
                                &MarkdownBlockKind::Table,
                                &lines,
                                BoundaryHint::TableEnd,
                            );
                            let rendered = render_block(&MarkdownBlockKind::Table, &lines);
                            self.state = ClassifierState::Plain;
                            decisions.push(BlockDecision::FinishHold {
                                status,
                                kind: MarkdownBlockKind::Table,
                                lines: rendered,
                            });
                            next = Some(line);
                        }
                    }
                    MarkdownBlockKind::List => {
                        if is_list_item(&line) {
                            lines.push(line);
                            let status = hold_status(
                                &MarkdownBlockKind::List,
                                &lines,
                                BoundaryHint::NonListLine,
                            );
                            self.state = ClassifierState::Holding {
                                kind: MarkdownBlockKind::List,
                                lines,
                                fence_marker: None,
                            };
                            decisions.push(BlockDecision::ContinueHold { status });
                        } else {
                            let status = hold_status(
                                &MarkdownBlockKind::List,
                                &lines,
                                BoundaryHint::NonListLine,
                            );
                            let rendered = render_block(&MarkdownBlockKind::List, &lines);
                            self.state = ClassifierState::Plain;
                            decisions.push(BlockDecision::FinishHold {
                                status,
                                kind: MarkdownBlockKind::List,
                                lines: rendered,
                            });
                            next = Some(line);
                        }
                    }
                    MarkdownBlockKind::Quote => {
                        if is_quote_line(&line) {
                            lines.push(line);
                            let status = hold_status(
                                &MarkdownBlockKind::Quote,
                                &lines,
                                BoundaryHint::NonQuoteLine,
                            );
                            self.state = ClassifierState::Holding {
                                kind: MarkdownBlockKind::Quote,
                                lines,
                                fence_marker: None,
                            };
                            decisions.push(BlockDecision::ContinueHold { status });
                        } else {
                            let status = hold_status(
                                &MarkdownBlockKind::Quote,
                                &lines,
                                BoundaryHint::NonQuoteLine,
                            );
                            let rendered = render_block(&MarkdownBlockKind::Quote, &lines);
                            self.state = ClassifierState::Plain;
                            decisions.push(BlockDecision::FinishHold {
                                status,
                                kind: MarkdownBlockKind::Quote,
                                lines: rendered,
                            });
                            next = Some(line);
                        }
                    }
                },
            }
        }

        // All held states share the same budget, including first lines, candidate
        // table headers, lists and quotes. Never retain oversized source between calls.
        let exceeded = match &self.state {
            ClassifierState::Holding { lines, .. } => held_block_too_large(lines),
            ClassifierState::PendingTableHeader(line) => line.len() > MAX_HELD_BYTES,
            ClassifierState::Plain => false,
        };
        if exceeded {
            let (kind, lines, hint) = match std::mem::take(&mut self.state) {
                ClassifierState::Holding { kind, lines, .. } => {
                    let hint = match kind {
                        MarkdownBlockKind::CodeFence => BoundaryHint::CodeFenceClose,
                        MarkdownBlockKind::Table => BoundaryHint::TableEnd,
                        MarkdownBlockKind::List => BoundaryHint::NonListLine,
                        MarkdownBlockKind::Quote => BoundaryHint::NonQuoteLine,
                    };
                    (kind, lines, hint)
                }
                ClassifierState::PendingTableHeader(line) => (
                    MarkdownBlockKind::Table,
                    vec![line],
                    BoundaryHint::TableSeparator,
                ),
                ClassifierState::Plain => return decisions,
            };
            if matches!(
                decisions.last(),
                Some(BlockDecision::StartHold { .. } | BlockDecision::ContinueHold { .. })
            ) {
                decisions.pop();
            }
            decisions.push(BlockDecision::FallbackImmediate {
                status: hold_status(&kind, &lines, hint),
                kind,
                reason: FallbackReason::HeldBlockTooLarge,
                lines,
            });
        }
        decisions
    }

    /// Flush buffered input, falling back for incomplete fences.
    pub fn finish(&mut self) -> Vec<BlockDecision> {
        match std::mem::take(&mut self.state) {
            ClassifierState::Plain => Vec::new(),
            ClassifierState::PendingTableHeader(header) => {
                vec![BlockDecision::ImmediateLine(header)]
            }
            ClassifierState::Holding { kind, lines, .. } => {
                let hint = match kind {
                    MarkdownBlockKind::CodeFence => BoundaryHint::CodeFenceClose,
                    MarkdownBlockKind::Table => BoundaryHint::TableEnd,
                    MarkdownBlockKind::List => BoundaryHint::NonListLine,
                    MarkdownBlockKind::Quote => BoundaryHint::NonQuoteLine,
                };
                let status = hold_status(&kind, &lines, hint);
                if kind == MarkdownBlockKind::CodeFence {
                    vec![BlockDecision::FallbackImmediate {
                        status,
                        kind: MarkdownBlockKind::CodeFence,
                        reason: FallbackReason::UnterminatedCodeFence,
                        lines,
                    }]
                } else {
                    vec![BlockDecision::FinishHold {
                        status,
                        kind: kind.clone(),
                        lines: render_block(&kind, &lines),
                    }]
                }
            }
        }
    }

    /// Discard buffered state after an explicit stream reset.
    pub fn reset(&mut self) {
        self.state = ClassifierState::Plain;
    }

    fn handle_plain(&mut self, line: String, decisions: &mut Vec<BlockDecision>) {
        if let Some(marker) = fence_marker(&line) {
            let lines = vec![line];
            let status = hold_status(
                &MarkdownBlockKind::CodeFence,
                &lines,
                BoundaryHint::CodeFenceClose,
            );
            self.state = ClassifierState::Holding {
                kind: MarkdownBlockKind::CodeFence,
                lines,
                fence_marker: Some(marker),
            };
            decisions.push(BlockDecision::StartHold { status });
        } else if is_possible_table_header(&line) {
            let status = HoldStatus {
                kind: MarkdownBlockKind::Table,
                lines: 1,
                bytes: line.len(),
                boundary_hint: BoundaryHint::TableSeparator,
            };
            self.state = ClassifierState::PendingTableHeader(line);
            decisions.push(BlockDecision::StartHold { status });
        } else if is_list_item(&line) {
            let lines = vec![line];
            let status = hold_status(&MarkdownBlockKind::List, &lines, BoundaryHint::NonListLine);
            self.state = ClassifierState::Holding {
                kind: MarkdownBlockKind::List,
                lines,
                fence_marker: None,
            };
            decisions.push(BlockDecision::StartHold { status });
        } else if is_quote_line(&line) {
            let lines = vec![line];
            let status = hold_status(
                &MarkdownBlockKind::Quote,
                &lines,
                BoundaryHint::NonQuoteLine,
            );
            self.state = ClassifierState::Holding {
                kind: MarkdownBlockKind::Quote,
                lines,
                fence_marker: None,
            };
            decisions.push(BlockDecision::StartHold { status });
        } else {
            self.state = ClassifierState::Plain;
            decisions.push(BlockDecision::ImmediateLine(line));
        }
    }
}

fn hold_status(
    kind: &MarkdownBlockKind,
    lines: &[String],
    boundary_hint: BoundaryHint,
) -> HoldStatus {
    HoldStatus {
        kind: kind.clone(),
        lines: lines.len(),
        bytes: lines.iter().map(String::len).sum(),
        boundary_hint,
    }
}

fn held_block_too_large(lines: &[String]) -> bool {
    lines.len() > MAX_HELD_LINES || lines.iter().map(String::len).sum::<usize>() > MAX_HELD_BYTES
}

fn render_block(kind: &MarkdownBlockKind, lines: &[String]) -> Vec<String> {
    match kind {
        MarkdownBlockKind::Table
        | MarkdownBlockKind::CodeFence
        | MarkdownBlockKind::List
        | MarkdownBlockKind::Quote => lines.to_vec(),
    }
}

fn fence_marker(line: &str) -> Option<(String, usize)> {
    let trimmed = line.trim_start();
    if let Some(rest) = trimmed.strip_prefix("```") {
        let count = 3 + rest.chars().take_while(|c| *c == '`').count();
        Some(("```".to_string(), count))
    } else if let Some(rest) = trimmed.strip_prefix("~~~") {
        let count = 3 + rest.chars().take_while(|c| *c == '~').count();
        Some(("~~~".to_string(), count))
    } else {
        None
    }
}

fn is_matching_fence_close(line: &str, marker: &str, open_count: usize) -> bool {
    let trimmed = line.trim_start();
    let marker_char = marker.chars().next().unwrap_or('`');
    let close_count = trimmed.chars().take_while(|c| *c == marker_char).count();
    if close_count < open_count {
        return false;
    }
    trimmed[close_count..].trim().is_empty()
}

fn is_possible_table_header(line: &str) -> bool {
    let trimmed = line.trim();
    let pipe_count = trimmed.chars().filter(|ch| *ch == '|').count();
    is_table_row(line)
        && pipe_count >= 2
        && line
            .split('|')
            .filter(|cell| !cell.trim().is_empty())
            .count()
            >= 2
}

fn is_table_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.contains('|') && !trimmed.is_empty()
}

fn is_table_separator(line: &str) -> bool {
    let trimmed = line.trim().trim_matches('|').trim();
    if trimmed.is_empty() {
        return false;
    }
    trimmed.split('|').all(|cell| {
        let cell = cell.trim();
        let cell = cell.trim_start_matches(':').trim_end_matches(':');
        cell.len() >= 3 && cell.chars().all(|ch| ch == '-')
    })
}

fn is_list_item(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("- ")
        || trimmed.starts_with("* ")
        || trimmed.starts_with("+ ")
        || ordered_list_prefix_len(trimmed).is_some()
}

fn ordered_list_prefix_len(line: &str) -> Option<usize> {
    let dot = line.find('.')?;
    if dot == 0 || !line[..dot].chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    line[dot + 1..].starts_with(' ').then_some(dot + 2)
}

fn is_quote_line(line: &str) -> bool {
    line.trim_start().starts_with("> ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines_from_finish(decisions: Vec<BlockDecision>) -> Vec<String> {
        decisions
            .into_iter()
            .flat_map(|decision| match decision {
                BlockDecision::ImmediateLine(line) => vec![line],
                BlockDecision::FinishHold { lines, .. }
                | BlockDecision::FallbackImmediate { lines, .. } => lines,
                BlockDecision::StartHold { .. } | BlockDecision::ContinueHold { .. } => Vec::new(),
            })
            .collect()
    }

    #[test]
    fn plain_text_emits_immediately() {
        let mut classifier = StreamBlockClassifier::default();
        assert_eq!(
            classifier.push_line("hello".to_string()),
            vec![BlockDecision::ImmediateLine("hello".to_string())]
        );
    }

    #[test]
    fn table_waits_for_separator_then_renders_aligned_rows() {
        let mut classifier = StreamBlockClassifier::default();
        assert!(matches!(
            classifier.push_line("| A | Longer |".to_string()).as_slice(),
            [BlockDecision::StartHold { status }]
                if status.boundary_hint == BoundaryHint::TableSeparator
        ));
        assert!(matches!(
            classifier.push_line("| --- | --- |".to_string()).as_slice(),
            [BlockDecision::StartHold { status }]
                if status.kind == MarkdownBlockKind::Table
                    && status.boundary_hint == BoundaryHint::TableEnd
        ));
        assert!(matches!(
            classifier.push_line("| x | yy |".to_string()).as_slice(),
            [BlockDecision::ContinueHold { status }]
                if status.lines == 3
        ));

        let decisions = classifier.push_line("after".to_string());
        assert_eq!(
            lines_from_finish(decisions),
            vec![
                "| A | Longer |".to_string(),
                "| --- | --- |".to_string(),
                "| x | yy |".to_string(),
                "after".to_string(),
            ]
        );
    }

    #[test]
    fn table_candidate_without_separator_reprocesses_current_line() {
        let mut classifier = StreamBlockClassifier::default();
        assert!(matches!(
            classifier.push_line("| maybe | table |".to_string()).as_slice(),
            [BlockDecision::StartHold { status }]
                if status.boundary_hint == BoundaryHint::TableSeparator
        ));

        assert_eq!(
            classifier.push_line("plain".to_string()),
            vec![
                BlockDecision::ImmediateLine("| maybe | table |".to_string()),
                BlockDecision::ImmediateLine("plain".to_string()),
            ]
        );
    }

    #[test]
    fn code_fence_suppresses_table_detection() {
        let mut classifier = StreamBlockClassifier::default();
        assert!(matches!(
            classifier.push_line("```".to_string()).as_slice(),
            [BlockDecision::StartHold { status }]
                if status.kind == MarkdownBlockKind::CodeFence
        ));
        assert!(matches!(
            classifier.push_line("| not | table |".to_string()).as_slice(),
            [BlockDecision::ContinueHold { status }]
                if status.kind == MarkdownBlockKind::CodeFence
        ));

        let decisions = classifier.push_line("```".to_string());
        assert_eq!(
            lines_from_finish(decisions),
            vec![
                "```".to_string(),
                "| not | table |".to_string(),
                "```".to_string()
            ]
        );
    }

    #[test]
    fn unterminated_code_fence_falls_back_on_finish() {
        let mut classifier = StreamBlockClassifier::default();
        let _ = classifier.push_line("```rust".to_string());
        let _ = classifier.push_line("fn main() {}".to_string());

        assert!(matches!(
            classifier.finish().as_slice(),
            [BlockDecision::FallbackImmediate { reason, lines, .. }]
                if *reason == FallbackReason::UnterminatedCodeFence
                    && lines.len() == 2
        ));
    }

    #[test]
    fn fence_info_string_not_treated_as_close() {
        let mut classifier = StreamBlockClassifier::default();
        let _ = classifier.push_line("```text".to_string());
        let _ = classifier.push_line("some code".to_string());

        let decisions = classifier.push_line("```rust".to_string());
        assert!(
            decisions
                .iter()
                .all(|d| !matches!(d, BlockDecision::FinishHold { .. })),
            "info-string line ```rust should NOT close the fence"
        );

        let decisions = classifier.push_line("fn main() {}".to_string());
        assert!(
            decisions
                .iter()
                .all(|d| !matches!(d, BlockDecision::FinishHold { .. })),
            "content after non-close should still be held"
        );

        let decisions = classifier.push_line("```".to_string());
        assert!(
            decisions
                .iter()
                .any(|d| matches!(d, BlockDecision::FinishHold { .. })),
            "bare ``` should close the fence"
        );
    }

    #[test]
    fn fence_nested_backtick_count() {
        let mut classifier = StreamBlockClassifier::default();
        let _ = classifier.push_line("````text".to_string());
        let _ = classifier.push_line("```rust".to_string());
        let _ = classifier.push_line("fn main() {}".to_string());

        let decisions = classifier.push_line("```".to_string());
        assert!(
            decisions
                .iter()
                .all(|d| !matches!(d, BlockDecision::FinishHold { .. })),
            "3-backtick line should NOT close 4-backtick fence"
        );

        let decisions = classifier.push_line("````".to_string());
        assert!(
            decisions
                .iter()
                .any(|d| matches!(d, BlockDecision::FinishHold { .. })),
            "4-backtick line should close 4-backtick fence"
        );
    }
}
