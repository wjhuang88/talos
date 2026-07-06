//! TUI widgets and render helpers.

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use talos_core::ApprovalChoice;
use talos_core::tool::ToolProvenance;

use crate::theme::semantic;

/// Maximum length for tool result content before truncation.
const MAX_RESULT_LENGTH: usize = 200;

/// Renders a tool call as a styled bubble in the chat viewport.
///
/// Displays the tool name in bold with accent color, truncated arguments,
/// and a result status indicator when available.
pub struct ToolCallBubble<'a> {
    /// Name of the tool.
    pub(crate) tool_name: &'a str,
    /// Formatted arguments (may be truncated).
    pub(crate) arguments: &'a str,
    /// Whether the tool result was an error.
    pub(crate) result_status: Option<bool>,
    /// Result content (may be truncated).
    pub(crate) result_content: Option<&'a str>,
    /// Origin of the tool.
    pub(crate) provenance: ToolProvenance,
}

impl<'a> ToolCallBubble<'a> {
    /// Creates a new tool call bubble with the given tool name and arguments.
    pub fn new(tool_name: &'a str, arguments: &'a str) -> Self {
        Self {
            tool_name,
            arguments,
            result_status: None,
            result_content: None,
            provenance: ToolProvenance::Native,
        }
    }

    /// Sets the provenance marker for this bubble.
    pub fn with_provenance(mut self, provenance: ToolProvenance) -> Self {
        self.provenance = provenance;
        self
    }

    /// Sets the result status and content for this bubble.
    pub fn with_result(mut self, is_error: bool, content: &'a str) -> Self {
        self.result_status = Some(is_error);
        self.result_content = Some(content);
        self
    }
}

impl ratatui::widgets::Widget for ToolCallBubble<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let mut lines: Vec<Line<'static>> = Vec::new();

        let prefix_style = Style::default()
            .fg(semantic::PREFIX_ASSISTANT)
            .add_modifier(Modifier::BOLD);
        let tool_name_style = Style::default()
            .fg(semantic::TEXT_ACCENT)
            .add_modifier(Modifier::BOLD);
        let dim_style = Style::default().fg(semantic::DIM_TEXT);
        let mut spans = vec![
            Span::styled(" → ", prefix_style),
            Span::styled(self.tool_name.to_string(), tool_name_style),
        ];
        let provenance_badge: Option<String> = match &self.provenance {
            ToolProvenance::Native => None,
            ToolProvenance::McpRemote { server } => Some(format!("[mcp:{}]", truncate(server, 24))),
            ToolProvenance::Plugin {
                name,
                version,
                carrier,
            } => Some(format!("[plugin:{}@{}/{}]", name, version, carrier)),
        };
        if let Some(badge) = provenance_badge.as_ref() {
            spans.push(Span::styled(badge.clone(), dim_style));
        }
        spans.push(Span::raw(", "));
        let prefix_width = 3
            + self.tool_name.chars().count()
            + provenance_badge
                .as_ref()
                .map(|badge| badge.chars().count())
                .unwrap_or(0)
            + 2;
        let args_budget = area.width as usize;
        let args_budget = args_budget.saturating_sub(prefix_width).max(1);
        let args_display = truncate(self.arguments, args_budget);
        spans.push(Span::styled(args_display, dim_style));
        lines.push(Line::from(spans));

        if let Some(is_error) = self.result_status {
            let (icon, style) = if is_error {
                ("✗ error", Style::default().fg(semantic::TEXT_ERROR))
            } else {
                ("✓ success", Style::default().fg(semantic::TEXT_SUCCESS))
            };
            lines.push(Line::from(Span::styled(format!("  {icon}"), style)));

            if let Some(content) = self.result_content {
                if let Some(diff_lines) = render_diff(content) {
                    lines.extend(diff_lines);
                } else {
                    let content_style = Style::default().fg(semantic::TEXT_PRIMARY);
                    let content_display = truncate(content, MAX_RESULT_LENGTH);
                    lines.push(Line::from(Span::styled(
                        format!("  {content_display}"),
                        content_style,
                    )));
                }
            }
        }

        let paragraph = Paragraph::new(Text::from(lines))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(semantic::BORDER_DEFAULT)),
            )
            .wrap(Wrap { trim: false });

        paragraph.render(area, buf);
    }
}

// ── Approval Overlay ─────────────────────────────────────────────────────────

/// Renders a semi-transparent approval overlay on top of the chat viewport.
///
/// Displays the tool name, arguments, risk level, and three options:
/// `[y] Approve once`, `[a] Always approve`, `[n] Deny`.
/// The currently selected option is highlighted with nord8.
pub struct ApprovalOverlay<'a> {
    /// Name of the tool requiring approval.
    tool_name: &'a str,
    /// Formatted arguments for the tool.
    arguments: &'a str,
    /// Currently selected choice.
    selected: &'a ApprovalChoice,
}

impl<'a> ApprovalOverlay<'a> {
    /// Creates a new approval overlay.
    pub fn new(tool_name: &'a str, arguments: &'a str, selected: &'a ApprovalChoice) -> Self {
        Self {
            tool_name,
            arguments,
            selected,
        }
    }
}

impl ratatui::widgets::Widget for ApprovalOverlay<'_> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let overlay_width = 50.min(area.width);
        let overlay_height = 10.min(area.height);
        let x = area.x + (area.width.saturating_sub(overlay_width)) / 2;
        let y = area.y + (area.height.saturating_sub(overlay_height)) / 2;
        let overlay_area = Rect {
            x,
            y,
            width: overlay_width,
            height: overlay_height,
        };

        Clear.render(overlay_area, buf);

        let mut lines: Vec<Line<'static>> = Vec::new();

        let title_style = Style::default()
            .fg(semantic::TEXT_SECONDARY_ACCENT)
            .add_modifier(Modifier::BOLD);
        lines.push(Line::from(Span::styled(
            "⚠ Permission Required",
            title_style,
        )));
        lines.push(Line::from(""));

        let tool_style = Style::default()
            .fg(semantic::TEXT_ACCENT)
            .add_modifier(Modifier::BOLD);
        lines.push(Line::from(Span::styled(
            format!("Tool: {}", self.tool_name),
            tool_style,
        )));

        let args_style = Style::default()
            .fg(semantic::DIM_TEXT)
            .add_modifier(Modifier::DIM);
        let args_budget = area.width.saturating_sub(8) as usize;
        let args_display = truncate(self.arguments, args_budget.max(1));
        lines.push(Line::from(Span::styled(
            format!("Args: {args_display}"),
            args_style,
        )));
        lines.push(Line::from(""));

        let risk_style = Style::default().fg(semantic::TEXT_WARNING);
        lines.push(Line::from(Span::styled(
            "Risk: Requires user approval",
            risk_style,
        )));
        lines.push(Line::from(""));

        let options = [
            ("y", "Approve once", ApprovalChoice::ApproveOnce),
            ("a", "Always approve", ApprovalChoice::AlwaysApprove),
            ("n", "Deny", ApprovalChoice::Deny),
        ];

        for (key, label, choice) in options {
            let style = if *self.selected == choice {
                Style::default()
                    .fg(semantic::TEXT_ACCENT)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(semantic::TEXT_PRIMARY)
            };
            lines.push(Line::from(Span::styled(format!("[{key}] {label}"), style)));
        }

        let paragraph = Paragraph::new(Text::from(lines)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(semantic::BORDER_ACCENT))
                .title(" Approval "),
        );

        paragraph.render(overlay_area, buf);
    }
}

/// Truncates a string to the given maximum length, appending "…" if truncated.
pub(crate) fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_len.saturating_sub(1)).collect();
        format!("{truncated}…")
    }
}

/// Detects unified diff content and renders it with color-coded lines.
///
/// Returns `Some` when the content appears to be a unified diff, `None` otherwise.
/// Detection heuristic: content contains lines starting with `diff --git`, `@@`,
/// or starting with `--- ` / `+++ ` (covers `a/`/`b/` git paths, `/dev/null`
/// new/deleted-file markers, and plain non-git unified diff headers).
pub(crate) fn render_diff(content: &str) -> Option<Vec<Line<'static>>> {
    let mut is_diff = false;
    for line in content.lines() {
        if line.starts_with("diff --git")
            || line.starts_with("@@")
            || line.starts_with("--- ")
            || line.starts_with("+++ ")
        {
            is_diff = true;
            break;
        }
    }
    if !is_diff {
        return None;
    }

    let mut lines: Vec<Line<'static>> = Vec::new();
    for line in content.lines() {
        let styled_line = if line.starts_with("diff --git")
            || line.starts_with("--- ")
            || line.starts_with("+++ ")
        {
            Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(semantic::TEXT_SECONDARY_ACCENT)
                    .add_modifier(Modifier::BOLD),
            ))
        } else if line.starts_with("@@") {
            Line::from(Span::styled(
                line.to_string(),
                Style::default().fg(semantic::TEXT_ACCENT),
            ))
        } else if is_diff_metadata_line(line) {
            Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(semantic::TEXT_WARNING)
                    .add_modifier(Modifier::ITALIC),
            ))
        } else if line.starts_with('+') && !line.starts_with("+++") {
            Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(semantic::TEXT_SUCCESS)
                    .bg(semantic::DIFF_ADDED_BG),
            ))
        } else if line.starts_with('-') && !line.starts_with("---") {
            Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(semantic::TEXT_ERROR)
                    .bg(semantic::DIFF_REMOVED_BG),
            ))
        } else {
            Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(semantic::TEXT_PRIMARY)
                    .add_modifier(Modifier::DIM),
            ))
        };
        lines.push(styled_line);
    }

    Some(lines)
}

/// Matches git diff metadata lines that are neither file/hunk headers nor
/// +/- content: index, mode changes, renames, copies, binary markers, and
/// the "no newline at end of file" marker.
fn is_diff_metadata_line(line: &str) -> bool {
    line.starts_with("index ")
        || line.starts_with("new file mode ")
        || line.starts_with("deleted file mode ")
        || line.starts_with("old mode ")
        || line.starts_with("new mode ")
        || line.starts_with("rename from ")
        || line.starts_with("rename to ")
        || line.starts_with("copy from ")
        || line.starts_with("copy to ")
        || line.starts_with("similarity index ")
        || line.starts_with("Binary files ")
        || line.starts_with("\\ No newline at end of file")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_text(line: &Line<'static>) -> String {
        line.spans.iter().map(|s| s.content.as_ref()).collect()
    }

    fn line_style(line: &Line<'static>) -> Style {
        line.spans[0].style
    }

    #[test]
    fn render_diff_returns_none_for_plain_text() {
        assert!(render_diff("just some regular tool output\nno diff markers here").is_none());
    }

    #[test]
    fn render_diff_does_not_false_positive_on_prose_mentioning_at_signs() {
        // A tool result quoting "@@" or "---" in prose (not real diff syntax)
        // should not be misclassified — this only holds when none of the
        // stronger anchors (diff --git, --- , +++ ) are present.
        assert!(render_diff("see the @@ decorator in Python").is_none());
    }

    #[test]
    fn render_diff_detects_git_diff_header() {
        let content = "diff --git a/src/lib.rs b/src/lib.rs\n@@ -1,2 +1,2 @@\n-old\n+new";
        assert!(render_diff(content).is_some());
    }

    #[test]
    fn render_diff_detects_dev_null_new_file_marker() {
        // git's new-file diff uses `--- /dev/null`, not `--- a/...` — the
        // original detection (`--- a/`) missed this.
        let content =
            "diff --git a/new.rs b/new.rs\n--- /dev/null\n+++ b/new.rs\n@@ -0,0 +1 @@\n+content";
        assert!(render_diff(content).is_some());
    }

    #[test]
    fn render_diff_detects_dev_null_deleted_file_marker() {
        let content =
            "diff --git a/old.rs b/old.rs\n--- a/old.rs\n+++ /dev/null\n@@ -1 +0,0 @@\n-content";
        assert!(render_diff(content).is_some());
    }

    #[test]
    fn render_diff_detects_non_git_unified_diff() {
        // Plain `diff -u` output has no `a/`/`b/` prefix at all.
        let content = "--- file.txt\n+++ file.txt\n@@ -1 +1 @@\n-old\n+new";
        assert!(render_diff(content).is_some());
    }

    #[test]
    fn render_diff_detects_bare_hunk_header() {
        let content = "@@ -1,3 +1,3 @@\n context\n-old\n+new";
        assert!(render_diff(content).is_some());
    }

    #[test]
    fn render_diff_styles_added_and_removed_lines() {
        let content = "@@ -1 +1 @@\n+added line\n-removed line";
        let lines = render_diff(content).unwrap();
        assert_eq!(line_text(&lines[1]), "+added line");
        assert_eq!(line_style(&lines[1]).fg, Some(semantic::TEXT_SUCCESS));
        assert_eq!(line_style(&lines[1]).bg, Some(semantic::DIFF_ADDED_BG));
        assert_eq!(line_text(&lines[2]), "-removed line");
        assert_eq!(line_style(&lines[2]).fg, Some(semantic::TEXT_ERROR));
        assert_eq!(line_style(&lines[2]).bg, Some(semantic::DIFF_REMOVED_BG));
    }

    #[test]
    fn render_diff_does_not_style_file_headers_as_added_or_removed() {
        let content = "--- a/x\n+++ b/x\n@@ -1 +1 @@\n-old\n+new";
        let lines = render_diff(content).unwrap();
        // `---`/`+++` headers must not fall into the +/- content branches.
        assert_ne!(line_style(&lines[0]).fg, Some(semantic::TEXT_ERROR));
        assert_ne!(line_style(&lines[1]).fg, Some(semantic::TEXT_SUCCESS));
        assert_eq!(
            line_style(&lines[0]).fg,
            Some(semantic::TEXT_SECONDARY_ACCENT)
        );
    }

    #[test]
    fn render_diff_styles_metadata_lines_distinctly() {
        let content = "diff --git a/x b/x\nnew file mode 100644\nindex 000..111\n--- /dev/null\n+++ b/x\n@@ -0,0 +1 @@\n+x";
        let lines = render_diff(content).unwrap();
        assert_eq!(line_text(&lines[1]), "new file mode 100644");
        assert_eq!(line_style(&lines[1]).fg, Some(semantic::TEXT_WARNING));
        assert_eq!(line_text(&lines[2]), "index 000..111");
        assert_eq!(line_style(&lines[2]).fg, Some(semantic::TEXT_WARNING));
    }

    #[test]
    fn render_diff_recognizes_binary_and_rename_and_no_newline_markers() {
        assert!(is_diff_metadata_line("Binary files a/x and b/x differ"));
        assert!(is_diff_metadata_line("rename from old_name.rs"));
        assert!(is_diff_metadata_line("rename to new_name.rs"));
        assert!(is_diff_metadata_line("similarity index 95%"));
        assert!(is_diff_metadata_line("\\ No newline at end of file"));
        assert!(!is_diff_metadata_line("+added content"));
        assert!(!is_diff_metadata_line("regular context line"));
    }

    #[test]
    fn render_diff_styles_context_lines_as_dim() {
        let content = "@@ -1 +1 @@\n unchanged context line";
        let lines = render_diff(content).unwrap();
        assert_eq!(line_text(&lines[1]), " unchanged context line");
        assert_eq!(line_style(&lines[1]).fg, Some(semantic::TEXT_PRIMARY));
        assert!(line_style(&lines[1]).add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn render_diff_preserves_full_realistic_new_file_diff() {
        let content = concat!(
            "diff --git a/greet.rs b/greet.rs\n",
            "new file mode 100644\n",
            "index 0000000..1111111\n",
            "--- /dev/null\n",
            "+++ b/greet.rs\n",
            "@@ -0,0 +1,2 @@\n",
            "+fn greet() {\n",
            "+}\n",
        );
        let lines = render_diff(content).unwrap();
        assert_eq!(
            lines.len(),
            8,
            "every input line must produce one output line"
        );
        assert_eq!(line_text(&lines[0]), "diff --git a/greet.rs b/greet.rs");
        assert_eq!(line_text(&lines[6]), "+fn greet() {");
        assert_eq!(line_style(&lines[6]).fg, Some(semantic::TEXT_SUCCESS));
    }
}
