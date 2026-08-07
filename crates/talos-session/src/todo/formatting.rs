//! Stable human-readable Todo output formatting.

use super::model::{TodoItem, TodoStatus};

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

/// One-line item summary: `[x] Title (priority) — full-uuid`.
fn format_item_inline(item: &TodoItem) -> String {
    format!(
        "{} {} ({}) — {}",
        status_icon(item.status),
        item.title,
        item.priority.as_str(),
        item.id,
    )
}

/// Multi-line item detail with description and tags (for create/update).
fn format_item_detail(item: &TodoItem) -> String {
    let mut text = format_item_inline(item);
    if let Some(desc) = &item.description
        && !desc.is_empty()
    {
        text.push_str(&format!("\n  description: {desc}"));
    }
    if !item.tags.is_empty() {
        text.push_str(&format!("\n  tags: {}", item.tags.join(", ")));
    }
    text
}

pub(super) fn format_created(item: &TodoItem) -> String {
    format!("Created: {}", format_item_detail(item))
}

pub(super) fn format_updated(item: &TodoItem) -> String {
    format!("Updated: {}", format_item_detail(item))
}

pub(super) fn format_query_result(items: &[TodoItem]) -> String {
    if items.is_empty() {
        return "No todos found.".to_string();
    }
    let mut text = format!("{} todo(s):", items.len());
    for item in items {
        text.push_str(&format!("\n  {}", format_item_inline(item)));
    }
    text
}

/// Builds a mutation result: action confirmation line + current active list.
///
/// Completed items are excluded so that finishing one list and starting a new
/// one does not keep old items in the output. Use `todo_query` with
/// `status: "completed"` to inspect finished items.
pub(super) fn format_mutation_result(action_text: &str, all_items: &[TodoItem]) -> String {
    let mut text = String::from(action_text);
    text.push_str("\n\n");
    let active: Vec<TodoItem> = all_items
        .iter()
        .filter(|i| i.status != TodoStatus::Completed)
        .cloned()
        .collect();
    text.push_str(&format_query_result(&active));
    text
}
