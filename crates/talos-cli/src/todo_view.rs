use anyhow::{Context, Result, anyhow, bail};
use talos_conversation::{
    MessageSource, StreamMessage, TodoCommandAction, TodoCommandRequest, TodoExportFormat,
    TodoPanelData, TodoPanelRow, UiOutput,
};
use tokio::sync::{mpsc, watch};
use uuid::Uuid;

pub(crate) fn handle_todo_command(
    ui_tx: &mpsc::UnboundedSender<UiOutput>,
    session_manager: &talos_session::SessionManager,
    session_watch_rx: &watch::Receiver<talos_session::Session>,
    req: TodoCommandRequest,
) {
    let session = session_watch_rx.borrow().clone();
    let repo = match session_manager.todo_repository() {
        Ok(repo) => repo,
        Err(error) => {
            send_stream(
                ui_tx,
                MessageSource::Error,
                format!("[Error] Failed to open todo repository: {error}\n"),
            );
            return;
        }
    };

    let query = match todo_query_from_request(&req) {
        Ok(query) => query,
        Err(message) => {
            send_stream(ui_tx, MessageSource::Error, format!("[Error] {message}\n"));
            return;
        }
    };

    match render_todo_view(&repo, session.id, req, query) {
        Ok(rendered) => {
            let _ = ui_tx.send(UiOutput::TodoPanel(rendered.panel));
            send_stream(ui_tx, MessageSource::System, rendered.text);
        }
        Err(error) => {
            send_stream(ui_tx, MessageSource::Error, format!("[Error] {error}\n"));
        }
    }
}

fn send_stream(ui_tx: &mpsc::UnboundedSender<UiOutput>, source: MessageSource, text: String) {
    let _ = ui_tx.send(UiOutput::Stream(StreamMessage {
        source,
        stream: Box::pin(futures::stream::once(async move { text })),
    }));
}

struct RenderedTodoView {
    panel: TodoPanelData,
    text: String,
}

fn todo_query_from_request(req: &TodoCommandRequest) -> Result<talos_session::TodoQuery, String> {
    Ok(talos_session::TodoQuery {
        status: match req.status_filter.as_deref() {
            None => None,
            Some("todo") => Some(talos_session::TodoStatus::Todo),
            Some("in_progress") | Some("active") => Some(talos_session::TodoStatus::InProgress),
            Some("completed") | Some("done") => Some(talos_session::TodoStatus::Completed),
            Some("blocked") => Some(talos_session::TodoStatus::Blocked),
            Some(other) => return Err(format!("Unknown todo status filter: {other}")),
        },
        priority: match req.priority_filter.as_deref() {
            None => None,
            Some("low") => Some(talos_session::TodoPriority::Low),
            Some("medium") => Some(talos_session::TodoPriority::Medium),
            Some("high") => Some(talos_session::TodoPriority::High),
            Some("critical") => Some(talos_session::TodoPriority::Critical),
            Some(other) => return Err(format!("Unknown todo priority filter: {other}")),
        },
        tag: req.tag_filter.clone(),
    })
}

fn render_todo_view(
    repo: &talos_session::TodoRepository,
    session_id: Uuid,
    req: TodoCommandRequest,
    query: talos_session::TodoQuery,
) -> Result<RenderedTodoView> {
    match req.action {
        TodoCommandAction::List => {
            let mut items = repo.list(session_id, query)?;
            sort_todo_items(&mut items, req.sort.as_deref())?;
            Ok(render_todo_list(&items))
        }
        TodoCommandAction::Show { id } => {
            let id = Uuid::parse_str(&id).context("invalid todo id")?;
            let item = repo
                .get(session_id, id)?
                .ok_or_else(|| anyhow!("todo item not found: {id}"))?;
            let dependencies = repo.list_dependencies(session_id)?;
            Ok(render_todo_show(&item, &dependencies))
        }
        TodoCommandAction::Stats => {
            let items = repo.list(session_id, talos_session::TodoQuery::default())?;
            Ok(render_todo_stats(&items))
        }
        TodoCommandAction::Export { format } => {
            let mut items = repo.list(session_id, query)?;
            sort_todo_items(&mut items, req.sort.as_deref())?;
            let dependencies = repo.list_dependencies(session_id)?;
            render_todo_export(&items, &dependencies, format)
        }
    }
}

fn sort_todo_items(items: &mut [talos_session::TodoItem], sort: Option<&str>) -> Result<()> {
    match sort.unwrap_or("priority") {
        "created" | "created_at" => items.sort_by_key(|item| item.created_at),
        "title" => items.sort_by(|a, b| a.title.cmp(&b.title)),
        "status" => items.sort_by_key(|item| todo_status_rank(item.status)),
        "priority" => items.sort_by_key(|item| todo_priority_rank(item.priority)),
        other => bail!("unknown todo sort: {other}"),
    }
    Ok(())
}

fn render_todo_list(items: &[talos_session::TodoItem]) -> RenderedTodoView {
    let mut text = String::from("[System] Session todos:\n");
    if items.is_empty() {
        text.push_str("[System]   (no todos)\n");
    } else {
        for item in items {
            text.push_str(&format!(
                "[System]   {} {} ({}) — {}\n",
                talos_session::status_icon(item.status),
                item.title,
                todo_priority_label(item.priority),
                short_id(item.id),
            ));
        }
    }
    RenderedTodoView {
        panel: TodoPanelData {
            title: "Session Todos".to_string(),
            rows: todo_panel_rows(items),
            footer: Some(format!("{} item{}", items.len(), plural(items.len()))),
        },
        text,
    }
}

fn render_todo_show(
    item: &talos_session::TodoItem,
    dependencies: &[talos_session::TodoDependency],
) -> RenderedTodoView {
    let parents: Vec<String> = dependencies
        .iter()
        .filter(|dep| dep.child_id == item.id)
        .map(|dep| dep.parent_id.to_string())
        .collect();
    let children: Vec<String> = dependencies
        .iter()
        .filter(|dep| dep.parent_id == item.id)
        .map(|dep| dep.child_id.to_string())
        .collect();
    let mut text = format!(
        "[System] Todo {} {}\n[System]   id: {}\n[System]   status: {}\n[System]   priority: {}\n",
        talos_session::status_icon(item.status),
        item.title,
        item.id,
        todo_status_label(item.status),
        todo_priority_label(item.priority)
    );
    if let Some(description) = &item.description {
        text.push_str(&format!("[System]   description: {description}\n"));
    }
    if !item.tags.is_empty() {
        text.push_str(&format!("[System]   tags: {}\n", item.tags.join(", ")));
    }
    if !parents.is_empty() {
        text.push_str(&format!("[System]   depends_on: {}\n", parents.join(", ")));
    }
    if !children.is_empty() {
        text.push_str(&format!("[System]   unlocks: {}\n", children.join(", ")));
    }
    RenderedTodoView {
        panel: TodoPanelData {
            title: "Todo Detail".to_string(),
            rows: todo_panel_rows(std::slice::from_ref(item)),
            footer: Some(item.id.to_string()),
        },
        text,
    }
}

fn render_todo_stats(items: &[talos_session::TodoItem]) -> RenderedTodoView {
    let mut counts = [0usize; 4];
    for item in items {
        counts[todo_status_rank(item.status)] += 1;
    }
    let text = format!(
        "[System] Todo stats: total={} todo={} in_progress={} blocked={} completed={}\n",
        items.len(),
        counts[0],
        counts[1],
        counts[2],
        counts[3],
    );
    RenderedTodoView {
        panel: TodoPanelData {
            title: "Todo Stats".to_string(),
            rows: Vec::new(),
            footer: Some(format!(
                "total {} | todo {} | active {} | blocked {} | done {}",
                items.len(),
                counts[0],
                counts[1],
                counts[2],
                counts[3]
            )),
        },
        text,
    }
}

fn render_todo_export(
    items: &[talos_session::TodoItem],
    dependencies: &[talos_session::TodoDependency],
    format: TodoExportFormat,
) -> Result<RenderedTodoView> {
    let text = match format {
        TodoExportFormat::Json => {
            let value = serde_json::json!({
                "items": items,
                "dependencies": dependencies,
            });
            format!(
                "[System] Todo export (json):\n{}\n",
                serde_json::to_string_pretty(&value)?
            )
        }
        TodoExportFormat::Markdown => {
            let mut text = String::from("[System] Todo export (markdown):\n# Session Todos\n\n");
            for item in items {
                text.push_str(&format!(
                    "- [{}] **{}** `{}` `{}` `{}`\n",
                    if item.status == talos_session::TodoStatus::Completed {
                        "x"
                    } else {
                        " "
                    },
                    item.title,
                    item.id,
                    todo_status_label(item.status),
                    todo_priority_label(item.priority)
                ));
            }
            if !dependencies.is_empty() {
                text.push_str("\n## Dependencies\n\n");
                for dep in dependencies {
                    text.push_str(&format!("- `{}` -> `{}`\n", dep.parent_id, dep.child_id));
                }
            }
            text
        }
    };
    Ok(RenderedTodoView {
        panel: TodoPanelData {
            title: "Todo Export".to_string(),
            rows: todo_panel_rows(items),
            footer: Some(match format {
                TodoExportFormat::Json => "json".to_string(),
                TodoExportFormat::Markdown => "markdown".to_string(),
            }),
        },
        text,
    })
}

fn todo_panel_rows(items: &[talos_session::TodoItem]) -> Vec<TodoPanelRow> {
    items
        .iter()
        .map(|item| TodoPanelRow {
            id: short_id(item.id),
            status: todo_status_label(item.status).to_string(),
            priority: todo_priority_label(item.priority).to_string(),
            title: item.title.clone(),
            detail: item.description.clone(),
        })
        .collect()
}

fn short_id(id: Uuid) -> String {
    id.to_string().chars().take(8).collect()
}

fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

fn todo_status_label(status: talos_session::TodoStatus) -> &'static str {
    match status {
        talos_session::TodoStatus::Todo => "todo",
        talos_session::TodoStatus::InProgress => "in_progress",
        talos_session::TodoStatus::Completed => "completed",
        talos_session::TodoStatus::Blocked => "blocked",
    }
}

fn todo_status_rank(status: talos_session::TodoStatus) -> usize {
    match status {
        talos_session::TodoStatus::Todo => 0,
        talos_session::TodoStatus::InProgress => 1,
        talos_session::TodoStatus::Blocked => 2,
        talos_session::TodoStatus::Completed => 3,
    }
}

fn todo_priority_label(priority: talos_session::TodoPriority) -> &'static str {
    match priority {
        talos_session::TodoPriority::Low => "low",
        talos_session::TodoPriority::Medium => "medium",
        talos_session::TodoPriority::High => "high",
        talos_session::TodoPriority::Critical => "critical",
    }
}

fn todo_priority_rank(priority: talos_session::TodoPriority) -> usize {
    match priority {
        talos_session::TodoPriority::Critical => 0,
        talos_session::TodoPriority::High => 1,
        talos_session::TodoPriority::Medium => 2,
        talos_session::TodoPriority::Low => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo_list_view_filters_and_sorts_items() {
        let dir = tempfile::tempdir().unwrap();
        let repo = talos_session::TodoRepository::new(&dir.path().join("todos.sqlite")).unwrap();
        repo.init_schema().unwrap();
        let session_id = Uuid::new_v4();
        repo.create(talos_session::CreateTodo {
            session_id,
            title: "medium item".to_string(),
            description: None,
            priority: talos_session::TodoPriority::Medium,
            assigned_to_turn: None,
            tags: vec!["ops".to_string()],
        })
        .unwrap();
        let high = repo
            .create(talos_session::CreateTodo {
                session_id,
                title: "high item".to_string(),
                description: Some("detail".to_string()),
                priority: talos_session::TodoPriority::High,
                assigned_to_turn: None,
                tags: vec!["ops".to_string()],
            })
            .unwrap();

        let rendered = render_todo_view(
            &repo,
            session_id,
            TodoCommandRequest {
                action: TodoCommandAction::List,
                status_filter: None,
                priority_filter: Some("high".to_string()),
                tag_filter: Some("ops".to_string()),
                sort: Some("priority".to_string()),
            },
            talos_session::TodoQuery {
                status: None,
                priority: Some(talos_session::TodoPriority::High),
                tag: Some("ops".to_string()),
            },
        )
        .unwrap();

        assert!(rendered.text.contains("high item"));
        assert!(!rendered.text.contains("medium item"));
        assert!(
            rendered.text.contains("[ ] high item (high)"),
            "expected checkbox-style rendering, got: {}",
            rendered.text
        );
        assert_eq!(rendered.panel.rows.len(), 1);
        assert_eq!(rendered.panel.rows[0].id, short_id(high.id));
    }

    #[test]
    fn todo_export_json_includes_dependencies() {
        let dir = tempfile::tempdir().unwrap();
        let repo = talos_session::TodoRepository::new(&dir.path().join("todos.sqlite")).unwrap();
        repo.init_schema().unwrap();
        let session_id = Uuid::new_v4();
        let parent = repo
            .create(talos_session::CreateTodo {
                session_id,
                title: "parent".to_string(),
                description: None,
                priority: talos_session::TodoPriority::High,
                assigned_to_turn: None,
                tags: vec![],
            })
            .unwrap();
        let child = repo
            .create(talos_session::CreateTodo {
                session_id,
                title: "child".to_string(),
                description: None,
                priority: talos_session::TodoPriority::Medium,
                assigned_to_turn: None,
                tags: vec![],
            })
            .unwrap();
        repo.add_dependency(session_id, parent.id, child.id)
            .unwrap();

        let rendered = render_todo_view(
            &repo,
            session_id,
            TodoCommandRequest {
                action: TodoCommandAction::Export {
                    format: TodoExportFormat::Json,
                },
                status_filter: None,
                priority_filter: None,
                tag_filter: None,
                sort: Some("created".to_string()),
            },
            talos_session::TodoQuery::default(),
        )
        .unwrap();

        assert!(rendered.text.contains("\"items\""));
        assert!(rendered.text.contains("\"dependencies\""));
        assert!(rendered.text.contains(&parent.id.to_string()));
        assert!(rendered.text.contains(&child.id.to_string()));
    }
}
