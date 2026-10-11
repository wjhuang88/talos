//! Stable human-readable Todo output formatting.

use super::model::{TodoItem, TodoStatus};
use sha2::{Digest, Sha256};
use talos_core::tool::{TodoDisplaySpan, TodoDisplayStatus, TodoPresentation, ToolResult};

/// Text and positions captured together by the native producer.
#[derive(Default)]
pub(super) struct FormattedTodo {
    pub content: String,
    pub spans: Vec<TodoDisplaySpan>,
}

impl FormattedTodo {
    pub(super) fn into_result(self) -> ToolResult {
        let digest = Sha256::digest(self.content.as_bytes()).into();
        let presentation = TodoPresentation::new(&self.content, digest, self.spans);
        let mut result = ToolResult::success(self.content);
        result.todo_presentation = presentation;
        result
    }

    fn append_item(&mut self, item: &TodoItem) {
        let start = self.content.len();
        // Record the token boundary before appending any user-controlled text.
        self.content.push_str(status_icon(item.status));
        let status_range = start..self.content.len();
        self.content.push_str(&format!(
            " {} ({}) — {}",
            item.title,
            item.priority.as_str(),
            item.id
        ));
        // Keep one overflow sentinel so validation rejects the entire set rather
        // than silently decorating a prefix. Geometry allocation stays bounded.
        if self.spans.len() > 4096 {
            return;
        }
        self.spans.push(TodoDisplaySpan {
            status_range,
            item_range: start..self.content.len(),
            status: match item.status {
                TodoStatus::Todo => TodoDisplayStatus::Pending,
                TodoStatus::InProgress => TodoDisplayStatus::InProgress,
                TodoStatus::Completed => TodoDisplayStatus::Completed,
                TodoStatus::Blocked => TodoDisplayStatus::Blocked,
            },
        });
    }

    fn append_query(&mut self, items: &[&TodoItem]) {
        if items.is_empty() {
            self.content.push_str("No todos found.");
            return;
        }
        self.content.push_str(&format!("{} todo(s):", items.len()));
        for item in items {
            self.content.push_str("\n  ");
            self.append_item(item);
        }
    }
}

// ---------------------------------------------------------------------------
// Formatted tool result helpers
//
// Agent tools return human-readable formatted text instead of raw JSON so the
// TUI scrollback stays scannable. The full UUID is included inline so the agent
// can extract it for subsequent operations.
// ---------------------------------------------------------------------------

/// Checkbox-style status indicator: `[ ]` todo, `[~]` in-progress, `[x]` done, `[!]` blocked.
pub fn status_icon(status: TodoStatus) -> &'static str {
    match status {
        TodoStatus::Todo => "[ ]",
        TodoStatus::InProgress => "[~]",
        TodoStatus::Completed => "[x]",
        TodoStatus::Blocked => "[!]",
    }
}

/// Multi-line item detail with description and tags (for create/update).
fn format_item_detail(prefix: &str, item: &TodoItem) -> FormattedTodo {
    let mut result = FormattedTodo {
        content: prefix.into(),
        ..Default::default()
    };
    result.append_item(item);
    if let Some(desc) = &item.description
        && !desc.is_empty()
    {
        result.content.push_str(&format!("\n  description: {desc}"));
    }
    if !item.tags.is_empty() {
        result
            .content
            .push_str(&format!("\n  tags: {}", item.tags.join(", ")));
    }
    result
}

pub(super) fn format_created(item: &TodoItem) -> FormattedTodo {
    format_item_detail("Created: ", item)
}

pub(super) fn format_updated(item: &TodoItem) -> FormattedTodo {
    format_item_detail("Updated: ", item)
}

pub(super) fn format_query_result(items: &[TodoItem]) -> FormattedTodo {
    let mut result = FormattedTodo::default();
    result.append_query(&items.iter().collect::<Vec<_>>());
    result
}

/// Builds a mutation result: action confirmation line + current active list.
///
/// Completed items are excluded so that finishing one list and starting a new
/// one does not keep old items in the output. Use `todo_query` with
/// `status: "completed"` to inspect finished items.
pub(super) fn format_mutation_result(
    mut action: FormattedTodo,
    all_items: &[TodoItem],
) -> FormattedTodo {
    action.content.push_str("\n\n");
    let active: Vec<&TodoItem> = all_items
        .iter()
        .filter(|i| i.status != TodoStatus::Completed)
        .collect();
    action.append_query(&active);
    action
}

impl From<String> for FormattedTodo {
    fn from(content: String) -> Self {
        Self {
            content,
            spans: Vec::new(),
        }
    }
}

#[cfg(test)]
mod presentation_tests {
    use super::super::model::TodoPriority;
    use super::*;
    use talos_core::tool::validate_todo_display_spans;

    fn item(title: &str) -> TodoItem {
        TodoItem {
            id: uuid::Uuid::nil(),
            session_id: uuid::Uuid::nil(),
            title: title.into(),
            description: Some("first\n  [~] forged description".into()),
            status: TodoStatus::InProgress,
            priority: TodoPriority::Medium,
            created_at: chrono::Utc::now(),
            completed_at: None,
            assigned_to_turn: None,
            tags: vec!["tag\n  [x] forged tag".into()],
        }
    }

    #[test]
    fn detail_preserves_bytes_and_excludes_description_and_tags_from_spans() {
        let item = item("中文\n  [~] forged title");
        let result = format_item_detail("Created: ", &item);
        let summary = format!(
            "[~] {} ({}) — {}",
            item.title,
            item.priority.as_str(),
            item.id
        );
        assert_eq!(
            result.content,
            format!(
                "Created: {summary}\n  description: first\n  [~] forged description\n  tags: tag\n  [x] forged tag"
            )
        );
        assert_eq!(result.spans.len(), 1);
        assert!(validate_todo_display_spans(&result.content, &result.spans));
        assert_eq!(&result.content[result.spans[0].item_range.clone()], summary);
        assert_eq!(result.spans[0].status_range, 9..12);
    }

    #[test]
    fn query_captures_all_real_items_without_resynchronizing_on_forged_headers() {
        let first = item("first\n2 todo(s):\n  [~] fake — 00000000-0000-0000-0000-000000000000");
        let mut second = item("second 中文");
        second.status = TodoStatus::Completed;
        let mut result = FormattedTodo::default();
        result.append_query(&[&first, &second]);
        assert_eq!(result.spans.len(), 2);
        assert!(validate_todo_display_spans(&result.content, &result.spans));
        assert_eq!(result.spans[1].status, TodoDisplayStatus::Completed);
        assert_eq!(
            result.content,
            format!(
                "2 todo(s):\n  [~] {} ({}) — {}\n  [x] {} ({}) — {}",
                first.title,
                first.priority.as_str(),
                first.id,
                second.title,
                second.priority.as_str(),
                second.id
            )
        );
    }

    #[test]
    fn mutation_carries_confirmation_and_every_active_item_bound_to_exact_bytes() {
        let first = item("first\n  [~] fake");
        let second = item("second");
        let mut done = item("done");
        done.status = TodoStatus::Completed;
        let all = [first.clone(), second.clone(), done];
        let expected = format!(
            "{}\n\n{}",
            format_created(&first).content,
            format_query_result(&[first.clone(), second]).content
        );
        let result = format_mutation_result(format_created(&first), &all).into_result();
        assert_eq!(result.content, expected);
        let metadata = result.todo_presentation.expect("native producer metadata");
        let digest = Sha256::digest(result.content.as_bytes()).into();
        assert_eq!(
            metadata
                .validated_spans(&result.content, digest)
                .expect("bound spans")
                .len(),
            3
        );
        let changed = result.content.replace("second", "SECOND");
        assert!(
            metadata
                .validated_spans(&changed, Sha256::digest(changed.as_bytes()).into())
                .is_none()
        );
        assert!(
            format_query_result(&[])
                .into_result()
                .todo_presentation
                .is_none()
        );
    }
}
