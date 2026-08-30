use super::*;
use rusqlite::OptionalExtension;
use talos_core::tool::{AgentTool, ToolNature};
use tempfile::tempdir;
use uuid::Uuid;

/// Extract the first UUID found in a formatted tool result string.
fn extract_uuid_from_text(text: &str) -> String {
    // UUID v4 format: 8-4-4-4-12 hex digits separated by dashes (36 chars total).
    // We scan for the dash positions as anchors since those are rare in prose.
    let bytes = text.as_bytes();
    for i in 0..bytes.len().saturating_sub(35) {
        if bytes[i + 8] == b'-'
            && bytes[i + 13] == b'-'
            && bytes[i + 18] == b'-'
            && bytes[i + 23] == b'-'
            && text[i..i + 36]
                .chars()
                .filter(|&c| c != '-')
                .all(|c| c.is_ascii_hexdigit())
        {
            return text[i..i + 36].to_string();
        }
    }
    String::new()
}

fn repo() -> TodoRepository {
    let dir = tempdir().expect("temp dir");
    let repo = TodoRepository::new(&dir.path().join("todos.sqlite")).expect("repo");
    repo.init_schema().expect("schema");
    repo
}

fn create(repo: &TodoRepository, session_id: Uuid, title: &str) -> TodoItem {
    repo.create(CreateTodo {
        session_id,
        title: title.to_string(),
        description: None,
        priority: TodoPriority::Medium,
        assigned_to_turn: None,
        tags: vec![],
    })
    .expect("create todo")
}

#[test]
fn create_and_get_round_trips_item() {
    let repo = repo();
    let session_id = Uuid::new_v4();

    let item = repo
        .create(CreateTodo {
            session_id,
            title: "Implement repository".to_string(),
            description: Some("SQLite CRUD".to_string()),
            priority: TodoPriority::High,
            assigned_to_turn: Some("turn-1".to_string()),
            tags: vec!["session".to_string(), " session ".to_string()],
        })
        .expect("create");

    let loaded = repo
        .get(session_id, item.id)
        .expect("get")
        .expect("item exists");
    assert_eq!(loaded.title, "Implement repository");
    assert_eq!(loaded.description.as_deref(), Some("SQLite CRUD"));
    assert_eq!(loaded.priority, TodoPriority::High);
    assert_eq!(loaded.assigned_to_turn.as_deref(), Some("turn-1"));
    assert_eq!(loaded.tags, vec!["session"]);
    assert_eq!(
        repo.list_work_units(session_id).expect("projection")[0]
            .identity
            .revision,
        1
    );
}

#[test]
fn updates_advance_canonical_revision_and_project_work_unit() {
    let repo = repo();
    let session_id = Uuid::new_v4();
    let item = create(&repo, session_id, "canonical");
    repo.update(
        session_id,
        item.id,
        TodoUpdate {
            title: Some("canonical v2".into()),
            ..TodoUpdate::default()
        },
    )
    .expect("update");
    let work = repo.list_work_units(session_id).expect("projection")[0].clone();
    assert_eq!(work.identity.id, item.id);
    assert_eq!(work.identity.revision, 2);
    assert_eq!(work.identity.kind, talos_core::work::WorkKind::WorkUnit);
}

#[test]
fn canonical_graph_snapshot_is_validated_and_consistent() {
    let repo = repo();
    let session_id = Uuid::new_v4();
    let first = create(&repo, session_id, "first");
    let second = create(&repo, session_id, "second");
    repo.add_dependency(session_id, first.id, second.id)
        .expect("dependency");
    let graph = repo.load_work_graph(session_id).expect("graph");
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn update_batch_rolls_back_every_item_when_a_later_item_is_missing() {
    let repo = repo();
    let session_id = Uuid::new_v4();
    let first = create(&repo, session_id, "first");
    let before = repo.load_work_graph(session_id).expect("before");
    let error = repo
        .update_batch(
            session_id,
            vec![
                (
                    first.id,
                    TodoUpdate {
                        title: Some("changed".into()),
                        ..TodoUpdate::default()
                    },
                ),
                (Uuid::new_v4(), TodoUpdate::default()),
            ],
        )
        .expect_err("batch must fail");
    assert!(matches!(error, TodoError::NotFound(_)));
    assert_eq!(
        repo.get(session_id, first.id)
            .expect("get")
            .expect("item")
            .title,
        "first"
    );
    assert_eq!(repo.load_work_graph(session_id).expect("after"), before);
}

#[test]
fn dependency_mutation_advances_dependent_revision_only_on_real_change() {
    let repo = repo();
    let session_id = Uuid::new_v4();
    let parent = create(&repo, session_id, "parent");
    let child = create(&repo, session_id, "child");
    repo.add_dependency(session_id, parent.id, child.id)
        .expect("add");
    let after_add = repo.load_work_graph(session_id).expect("after add");
    let child_after_add = after_add
        .nodes
        .iter()
        .find(|node| node.identity.id == child.id)
        .expect("child");
    assert_eq!(child_after_add.identity.revision, 2);
    repo.add_dependency(session_id, parent.id, child.id)
        .expect("idempotent add");
    assert_eq!(
        repo.load_work_graph(session_id).expect("duplicate"),
        after_add
    );
    assert!(
        repo.remove_dependency(session_id, parent.id, child.id)
            .expect("remove")
    );
    let after_remove = repo.load_work_graph(session_id).expect("after remove");
    assert_eq!(
        after_remove
            .nodes
            .iter()
            .find(|node| node.identity.id == child.id)
            .expect("child")
            .identity
            .revision,
        3
    );
    assert!(
        !repo
            .remove_dependency(session_id, parent.id, child.id)
            .expect("absent")
    );
    assert_eq!(
        repo.load_work_graph(session_id).expect("absent graph"),
        after_remove
    );
}

#[test]
fn readding_dependency_preserves_stable_identity_and_advances_edge_revision() {
    let repo = repo();
    let session_id = Uuid::new_v4();
    let parent = create(&repo, session_id, "parent");
    let child = create(&repo, session_id, "child");
    repo.add_dependency(session_id, parent.id, child.id)
        .expect("add");
    let first = repo.load_work_graph(session_id).expect("graph").edges[0];
    assert!(
        repo.remove_dependency(session_id, parent.id, child.id)
            .expect("remove")
    );
    repo.add_dependency(session_id, parent.id, child.id)
        .expect("re-add");
    let second = repo.load_work_graph(session_id).expect("graph").edges[0];
    assert_eq!(second.identity.id, first.identity.id);
    assert_eq!(second.identity.revision, first.identity.revision + 1);
}

#[test]
fn deleting_parent_advances_surviving_child_revision() {
    let repo = repo();
    let session_id = Uuid::new_v4();
    let parent = create(&repo, session_id, "parent");
    let child = create(&repo, session_id, "child");
    repo.add_dependency(session_id, parent.id, child.id)
        .expect("dependency");
    let before = repo
        .list_work_units(session_id)
        .expect("work units")
        .into_iter()
        .find(|node| node.identity.id == child.id)
        .expect("child work unit")
        .identity
        .revision;
    let mut repo = repo;
    assert!(repo.delete(session_id, parent.id).expect("delete"));
    let after = repo
        .list_work_units(session_id)
        .expect("work units")
        .into_iter()
        .find(|node| node.identity.id == child.id)
        .expect("child work unit")
        .identity
        .revision;
    assert_eq!(after, before + 1);
}

#[test]
fn legacy_schema_migrates_losslessly_and_keeps_backup() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("todos.sqlite");
    let session_id = Uuid::new_v4();
    let parent_id = Uuid::new_v4();
    let child_id = Uuid::new_v4();
    let connection = rusqlite::Connection::open(&path).expect("legacy database");
    connection
        .execute_batch(
            "CREATE TABLE todo_items (id TEXT PRIMARY KEY, session_id TEXT NOT NULL, \
             title TEXT NOT NULL, description TEXT, status TEXT NOT NULL, priority TEXT NOT NULL, \
             created_at TEXT NOT NULL, completed_at TEXT, assigned_to_turn TEXT, \
             tags_json TEXT NOT NULL DEFAULT '[]'); \
             CREATE TABLE todo_dependencies (session_id TEXT NOT NULL, parent_id TEXT NOT NULL, \
             child_id TEXT NOT NULL, PRIMARY KEY(session_id,parent_id,child_id));",
        )
        .expect("legacy schema");
    for (id, title) in [(parent_id, "parent"), (child_id, "child")] {
        connection
            .execute(
                "INSERT INTO todo_items VALUES (?1, ?2, ?3, NULL, 'todo', 'medium', \
             '2026-08-30T00:00:00Z', NULL, NULL, '[]')",
                rusqlite::params![id.to_string(), session_id.to_string(), title],
            )
            .expect("legacy item");
    }
    connection
        .execute(
            "INSERT INTO todo_dependencies VALUES (?1, ?2, ?3)",
            rusqlite::params![
                session_id.to_string(),
                parent_id.to_string(),
                child_id.to_string()
            ],
        )
        .expect("legacy edge");
    drop(connection);

    let repo = TodoRepository::new(&path).expect("repo");
    repo.init_schema().expect("migrate");
    let graph = repo.load_work_graph(session_id).expect("graph");
    assert_eq!(graph.nodes.len(), 2);
    assert!(graph.nodes.iter().all(|node| node.identity.revision == 1));
    assert_eq!(graph.edges.len(), 1);
    assert_eq!(graph.edges[0].identity.revision, 1);
    assert!(dir.path().join("todos.sqlite.pre-work-v1.bak").exists());
}

#[test]
fn lossy_legacy_value_fails_before_migration_or_backup() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("todos.sqlite");
    let connection = rusqlite::Connection::open(&path).expect("legacy database");
    connection.execute_batch(
        "CREATE TABLE todo_items (id TEXT PRIMARY KEY, session_id TEXT NOT NULL, title TEXT NOT NULL, \
         description TEXT, status TEXT NOT NULL, priority TEXT NOT NULL, created_at TEXT NOT NULL, \
         completed_at TEXT, assigned_to_turn TEXT, tags_json TEXT NOT NULL DEFAULT '[]'); \
         CREATE TABLE todo_dependencies (session_id TEXT NOT NULL, parent_id TEXT NOT NULL, \
         child_id TEXT NOT NULL, PRIMARY KEY(session_id,parent_id,child_id));",
    ).expect("legacy schema");
    connection
        .execute(
            "INSERT INTO todo_items VALUES (?1, ?2, 'bad', NULL, 'unknown', 'medium', \
         '2026-08-30T00:00:00Z', NULL, NULL, '[]')",
            rusqlite::params![Uuid::new_v4().to_string(), Uuid::new_v4().to_string()],
        )
        .expect("bad row");
    drop(connection);
    let repo = TodoRepository::new(&path).expect("repo");
    assert!(matches!(repo.init_schema(), Err(TodoError::Migration(_))));
    assert!(!dir.path().join("todos.sqlite.pre-work-v1.bak").exists());
}

#[test]
fn partial_schema_is_rejected_without_creating_the_missing_table() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("todos.sqlite");
    let connection = rusqlite::Connection::open(&path).expect("database");
    connection
        .execute_batch(
            "CREATE TABLE todo_items (id TEXT PRIMARY KEY, session_id TEXT NOT NULL, \
             title TEXT NOT NULL, description TEXT, status TEXT NOT NULL, priority TEXT NOT NULL, \
             created_at TEXT NOT NULL, completed_at TEXT, assigned_to_turn TEXT, \
             tags_json TEXT NOT NULL DEFAULT '[]');",
        )
        .expect("partial schema");
    drop(connection);
    let repo = TodoRepository::new(&path).expect("repo");
    assert!(matches!(repo.init_schema(), Err(TodoError::Migration(_))));
    let connection = rusqlite::Connection::open(&path).expect("database");
    let edge_table: Option<String> = connection
        .query_row(
            "SELECT type FROM sqlite_master WHERE name='todo_dependencies'",
            [],
            |row| row.get(0),
        )
        .optional()
        .expect("metadata");
    assert!(
        edge_table.is_none(),
        "migration must not create missing table"
    );
}

#[test]
fn duplicate_current_edge_identity_is_rejected() {
    let dir = tempdir().expect("temp dir");
    let path = dir.path().join("todos.sqlite");
    let connection = rusqlite::Connection::open(&path).expect("database");
    connection
        .execute_batch(
            "CREATE TABLE todo_items (id TEXT PRIMARY KEY, session_id TEXT NOT NULL, \
             title TEXT NOT NULL, description TEXT, status TEXT NOT NULL, priority TEXT NOT NULL, \
             created_at TEXT NOT NULL, completed_at TEXT, assigned_to_turn TEXT, \
             tags_json TEXT NOT NULL DEFAULT '[]', revision INTEGER NOT NULL DEFAULT 1); \
             CREATE TABLE todo_dependencies (session_id TEXT NOT NULL, parent_id TEXT NOT NULL, \
             child_id TEXT NOT NULL, edge_id TEXT NOT NULL, revision INTEGER NOT NULL DEFAULT 1, \
             PRIMARY KEY(session_id,parent_id,child_id));",
        )
        .expect("current schema");
    let session_id = Uuid::new_v4();
    let first = Uuid::new_v4();
    let second = Uuid::new_v4();
    let third = Uuid::new_v4();
    for (id, title) in [(first, "first"), (second, "second"), (third, "third")] {
        connection
            .execute(
                "INSERT INTO todo_items (id,session_id,title,status,priority,created_at,tags_json) \
                 VALUES (?1,?2,?3,'todo','medium','2026-08-30T00:00:00Z','[]')",
                rusqlite::params![id.to_string(), session_id.to_string(), title],
            )
            .expect("item");
    }
    let shared_edge_id = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO todo_dependencies VALUES (?1,?2,?3,?4,1)",
            rusqlite::params![
                session_id.to_string(),
                first.to_string(),
                second.to_string(),
                shared_edge_id
            ],
        )
        .expect("edge");
    connection
        .execute(
            "INSERT INTO todo_dependencies VALUES (?1,?2,?3,?4,1)",
            rusqlite::params![
                session_id.to_string(),
                second.to_string(),
                third.to_string(),
                shared_edge_id
            ],
        )
        .expect("edge");
    drop(connection);
    let repo = TodoRepository::new(&path).expect("repo");
    assert!(matches!(repo.init_schema(), Err(TodoError::Migration(_))));
}

#[test]
fn repository_projects_todos_and_edges_without_second_store() {
    let repo = repo();
    let session_id = Uuid::new_v4();
    let first = create(&repo, session_id, "first");
    let second = create(&repo, session_id, "second");
    repo.add_dependency(session_id, first.id, second.id)
        .expect("dependency");
    let units = repo.list_work_units(session_id).expect("units");
    let edges = repo.list_work_edges(session_id).expect("edges");
    assert_eq!(units.len(), 2);
    assert_eq!(
        edges,
        vec![talos_core::work::WorkEdge {
            identity: talos_core::work::WorkEdgeIdentity {
                id: {
                    use sha2::{Digest, Sha256};
                    let mut hasher = Sha256::new();
                    hasher.update(session_id.as_bytes());
                    hasher.update(first.id.as_bytes());
                    hasher.update(second.id.as_bytes());
                    let digest = hasher.finalize();
                    let mut bytes = [0_u8; 16];
                    bytes.copy_from_slice(&digest[..16]);
                    Uuid::from_bytes(bytes)
                },
                revision: 1,
            },
            parent_id: first.id,
            child_id: second.id
        }]
    );
}

#[test]
fn list_filters_by_status_priority_and_tag() {
    let repo = repo();
    let session_id = Uuid::new_v4();
    let other_session = Uuid::new_v4();
    let first = create(&repo, session_id, "first");
    let second = repo
        .create(CreateTodo {
            session_id,
            title: "second".to_string(),
            description: None,
            priority: TodoPriority::Critical,
            assigned_to_turn: None,
            tags: vec!["release".to_string()],
        })
        .expect("create second");
    create(&repo, other_session, "other");
    repo.update_status(session_id, first.id, TodoStatus::Completed)
        .expect("status");

    let results = repo
        .list(
            session_id,
            TodoQuery {
                status: Some(TodoStatus::Todo),
                priority: Some(TodoPriority::Critical),
                tag: Some("release".to_string()),
            },
        )
        .expect("list");

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, second.id);
}

#[test]
fn update_status_sets_and_clears_completed_at() {
    let repo = repo();
    let session_id = Uuid::new_v4();
    let item = create(&repo, session_id, "done");

    let completed = repo
        .update_status(session_id, item.id, TodoStatus::Completed)
        .expect("complete");
    assert!(completed.completed_at.is_some());

    let reopened = repo
        .update_status(session_id, item.id, TodoStatus::InProgress)
        .expect("reopen");
    assert!(reopened.completed_at.is_none());
}

#[test]
fn update_changes_optional_fields() {
    let repo = repo();
    let session_id = Uuid::new_v4();
    let item = create(&repo, session_id, "old");

    let updated = repo
        .update(
            session_id,
            item.id,
            TodoUpdate {
                title: Some("new".to_string()),
                description: Some(Some("details".to_string())),
                priority: Some(TodoPriority::Low),
                assigned_to_turn: Some(Some("turn-2".to_string())),
                tags: Some(vec!["b".to_string(), "a".to_string(), "b".to_string()]),
            },
        )
        .expect("update");

    assert_eq!(updated.title, "new");
    assert_eq!(updated.description.as_deref(), Some("details"));
    assert_eq!(updated.priority, TodoPriority::Low);
    assert_eq!(updated.assigned_to_turn.as_deref(), Some("turn-2"));
    assert_eq!(updated.tags, vec!["a", "b"]);
}

#[test]
fn delete_removes_item_and_dependency_edges() {
    let mut repo = repo();
    let session_id = Uuid::new_v4();
    let parent = create(&repo, session_id, "parent");
    let child = create(&repo, session_id, "child");
    repo.add_dependency(session_id, parent.id, child.id)
        .expect("dependency");

    assert!(repo.delete(session_id, parent.id).expect("delete"));
    assert!(
        repo.list_dependencies(session_id)
            .expect("dependencies")
            .is_empty()
    );
}

#[test]
fn dependency_cycle_is_rejected() {
    let repo = repo();
    let session_id = Uuid::new_v4();
    let first = create(&repo, session_id, "first");
    let second = create(&repo, session_id, "second");
    let third = create(&repo, session_id, "third");

    repo.add_dependency(session_id, first.id, second.id)
        .expect("first edge");
    repo.add_dependency(session_id, second.id, third.id)
        .expect("second edge");

    let err = repo
        .add_dependency(session_id, third.id, first.id)
        .expect_err("cycle");
    assert!(matches!(err, TodoError::DependencyCycle { .. }));
}

#[test]
fn dependency_requires_items_in_same_session() {
    let repo = repo();
    let session_id = Uuid::new_v4();
    let other_session = Uuid::new_v4();
    let parent = create(&repo, session_id, "parent");
    let child = create(&repo, other_session, "child");

    let err = repo
        .add_dependency(session_id, parent.id, child.id)
        .expect_err("missing child");
    assert!(matches!(err, TodoError::NotFound(id) if id == child.id));
}

#[test]
fn session_manager_opens_initialized_todo_repository() {
    let dir = tempdir().expect("temp dir");
    let manager = crate::SessionManager::with_dir(dir.path().to_path_buf());
    let repo = manager.todo_repository().expect("todo repository");
    let session_id = Uuid::new_v4();

    let item = create(&repo, session_id, "manager");

    assert_eq!(repo.db_path(), &dir.path().join("todos.sqlite"));
    assert!(repo.get(session_id, item.id).expect("get").is_some());
}

#[tokio::test]
async fn todo_tools_create_query_and_update_status() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("todos.sqlite");
    let session_id = Uuid::new_v4();
    let create_tool = TodoCreateTool::new(db_path.clone(), session_id);
    let query_tool = TodoQueryTool::new(db_path.clone(), session_id);
    let update_fields_tool = TodoUpdateTool::new(db_path.clone(), session_id);
    let add_dep_tool = TodoAddDependencyTool::new(db_path.clone(), session_id);
    let remove_dep_tool = TodoRemoveDependencyTool::new(db_path.clone(), session_id);
    let delete_tool = TodoDeleteTool::new(db_path.clone(), session_id);
    let update_tool = TodoUpdateStatusTool::new(db_path, session_id);

    // create — no session_id in the input; the tool supplies its own
    let created = create_tool
        .execute(serde_json::json!({
            "title": "tool item",
            "priority": "high",
            "tags": ["tool"]
        }))
        .await;
    assert!(!created.is_error, "{}", created.content);
    assert!(created.content.contains("Created:"));
    assert!(created.content.contains("tool item"));
    assert!(created.content.contains("(high)"));
    assert!(created.content.contains("[ ]")); // todo status
    let item_id = extract_uuid_from_text(&created.content);
    assert!(!item_id.is_empty());

    // query — returns formatted checklist
    let queried = query_tool
        .execute(serde_json::json!({
            "tag": "tool"
        }))
        .await;
    assert!(!queried.is_error, "{}", queried.content);
    assert!(queried.content.contains("1 todo(s):"));
    assert!(queried.content.contains("tool item"));
    assert!(queried.content.contains(&item_id));

    // update_status — returns formatted with [x] for completed
    let updated = update_tool
        .execute(serde_json::json!({
            "id": item_id,
            "status": "completed"
        }))
        .await;
    assert!(!updated.is_error, "{}", updated.content);
    assert!(updated.content.contains("Updated:"));
    assert!(updated.content.contains("[x]")); // completed status

    // update fields — title/priority/tags change
    let field_updated = update_fields_tool
        .execute(serde_json::json!({
            "id": item_id,
            "title": "renamed",
            "clear_description": true,
            "priority": "critical",
            "tags": ["next"]
        }))
        .await;
    assert!(!field_updated.is_error, "{}", field_updated.content);
    assert!(field_updated.content.contains("renamed"));
    assert!(field_updated.content.contains("(critical)"));

    // create a child for dependency tests
    let child = create_tool
        .execute(serde_json::json!({
            "title": "child"
        }))
        .await;
    assert!(!child.is_error, "{}", child.content);
    let child_id = extract_uuid_from_text(&child.content);

    // add dependency
    let dep = add_dep_tool
        .execute(serde_json::json!({
            "parent_id": item_id,
            "child_id": child_id
        }))
        .await;
    assert!(!dep.is_error, "{}", dep.content);
    assert!(dep.content.contains("Added dependency:"));

    // cycle detection
    let cycle = add_dep_tool
        .execute(serde_json::json!({
            "parent_id": child_id,
            "child_id": item_id
        }))
        .await;
    assert!(cycle.is_error);
    assert!(cycle.content.contains("cycle"));

    // remove dependency
    let removed = remove_dep_tool
        .execute(serde_json::json!({
            "parent_id": item_id,
            "child_id": child_id
        }))
        .await;
    assert!(!removed.is_error, "{}", removed.content);
    assert!(removed.content.contains("Removed dependency:"));

    // delete
    let deleted = delete_tool
        .execute(serde_json::json!({
            "id": child_id
        }))
        .await;
    assert!(!deleted.is_error, "{}", deleted.content);
    assert!(deleted.content.contains("Deleted todo item"));
}

#[tokio::test]
async fn todo_tool_ignores_session_id_if_model_sends_one() {
    // A model that hallucinates a stale/wrong session_id in its tool call
    // input must not be able to write into the wrong session. The tool
    // must use its constructor-bound session_id unconditionally.
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("todos.sqlite");
    let real_session = Uuid::new_v4();
    let wrong_session = Uuid::new_v4();
    let create_tool = TodoCreateTool::new(db_path.clone(), real_session);
    let query_tool = TodoQueryTool::new(db_path, real_session);

    let created = create_tool
        .execute(serde_json::json!({
            "session_id": wrong_session.to_string(),
            "title": "should land in real_session"
        }))
        .await;
    assert!(!created.is_error, "{}", created.content);

    let queried = query_tool.execute(serde_json::json!({})).await;
    assert!(queried.content.contains("should land in real_session"));
}

#[test]
fn todo_tools_expose_internal_permission_profiles() {
    let dir = tempdir().expect("temp dir");
    let session_id = Uuid::new_v4();
    let create_tool = TodoCreateTool::from_sessions_dir(dir.path(), session_id);
    let query_tool = TodoQueryTool::from_sessions_dir(dir.path(), session_id);

    let write_profile = create_tool.permission_profile(&serde_json::json!({}));
    let read_profile = query_tool.permission_profile(&serde_json::json!({}));

    assert_eq!(write_profile[0].nature, ToolNature::Internal);
    assert_eq!(read_profile[0].nature, ToolNature::Internal);
    let expected = format!("session:{session_id}:todos");
    assert_eq!(
        write_profile[0].resource.as_deref(),
        Some(expected.as_str())
    );
}

// --- TODO-002: Idempotent todo_create ---

#[test]
fn create_same_title_idempotent_same_session() {
    let repo = repo();
    let session_id = Uuid::new_v4();

    let first = repo
        .create(CreateTodo {
            session_id,
            title: "idempotent test".to_string(),
            description: Some("first description".to_string()),
            priority: TodoPriority::High,
            assigned_to_turn: None,
            tags: vec!["test".to_string()],
        })
        .expect("first create");

    // Second create with same session + title returns existing item.
    let second = repo
        .create(CreateTodo {
            session_id,
            title: "idempotent test".to_string(),
            description: None,
            priority: TodoPriority::Low,
            assigned_to_turn: None,
            tags: vec![],
        })
        .expect("second create");

    // Same id — no duplicate created.
    assert_eq!(first.id, second.id);
    // Original fields preserved (no merge/update).
    assert_eq!(second.description.as_deref(), Some("first description"));
    assert_eq!(second.priority, TodoPriority::High);
    assert_eq!(second.tags, vec!["test"]);

    // Only one row in the session.
    let all = repo.list_all(session_id).expect("list");
    assert_eq!(all.len(), 1);
}

#[test]
fn create_different_title_creates_new_item_same_session() {
    let repo = repo();
    let session_id = Uuid::new_v4();

    let first = create(&repo, session_id, "item one");
    let second = create(&repo, session_id, "item two");

    assert_ne!(first.id, second.id);
    let all = repo.list_all(session_id).expect("list");
    assert_eq!(all.len(), 2);
}

#[test]
fn create_same_title_different_session_not_deduped() {
    let repo = repo();
    let s1 = Uuid::new_v4();
    let s2 = Uuid::new_v4();

    let first = create(&repo, s1, "shared title");
    let second = create(&repo, s2, "shared title");

    // Different sessions — different items.
    assert_ne!(first.id, second.id);
    assert_eq!(first.title, second.title);

    // Each session has exactly one item.
    assert_eq!(repo.list_all(s1).expect("list").len(), 1);
    assert_eq!(repo.list_all(s2).expect("list").len(), 1);
}

#[tokio::test]
async fn todo_create_tool_idempotent_same_title() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("todos.sqlite");
    let session_id = Uuid::new_v4();
    let create_tool = TodoCreateTool::new(db_path, session_id);

    let first = create_tool
        .execute(serde_json::json!({
            "title": "idempotent tool test",
            "priority": "high",
            "tags": ["test"]
        }))
        .await;
    assert!(!first.is_error, "{}", first.content);
    let first_id = extract_uuid_from_text(&first.content);

    // Repeated create — same title, same session — returns existing.
    let second = create_tool
        .execute(serde_json::json!({
            "title": "idempotent tool test",
            "priority": "low"
        }))
        .await;
    assert!(!second.is_error, "{}", second.content);
    let second_id = extract_uuid_from_text(&second.content);

    assert_eq!(first_id, second_id);
    // Original priority preserved (high, not low).
    assert!(second.content.contains("(high)"));
}

// --- TODO-002: Batch create ---

#[test]
fn create_batch_creates_distinct_items() {
    let repo = repo();
    let session_id = Uuid::new_v4();

    let items = repo
        .create_batch(vec![
            CreateTodo {
                session_id,
                title: "alpha".to_string(),
                description: None,
                priority: TodoPriority::High,
                assigned_to_turn: None,
                tags: vec![],
            },
            CreateTodo {
                session_id,
                title: "beta".to_string(),
                description: None,
                priority: TodoPriority::Medium,
                assigned_to_turn: None,
                tags: vec![],
            },
        ])
        .expect("batch");

    assert_eq!(items.len(), 2);
    assert_ne!(items[0].id, items[1].id);
    assert_eq!(repo.list_all(session_id).expect("list").len(), 2);
}

#[test]
fn create_batch_deduplicates_same_title_within_batch() {
    let repo = repo();
    let session_id = Uuid::new_v4();

    let items = repo
        .create_batch(vec![
            CreateTodo {
                session_id,
                title: "shared".to_string(),
                description: Some("first".to_string()),
                priority: TodoPriority::High,
                assigned_to_turn: None,
                tags: vec![],
            },
            CreateTodo {
                session_id,
                title: "shared".to_string(),
                description: None,
                priority: TodoPriority::Low,
                assigned_to_turn: None,
                tags: vec![],
            },
        ])
        .expect("batch");

    assert_eq!(items.len(), 2);
    // Both results point to the same item (idempotent within batch).
    assert_eq!(items[0].id, items[1].id);
    // First-creation fields are preserved.
    assert_eq!(items[1].description.as_deref(), Some("first"));
    assert_eq!(items[1].priority, TodoPriority::High);
    assert_eq!(repo.list_all(session_id).expect("list").len(), 1);
}

#[test]
fn create_batch_deduplicates_against_existing_items() {
    let repo = repo();
    let session_id = Uuid::new_v4();

    let existing = create(&repo, session_id, "pre-existing");

    let items = repo
        .create_batch(vec![
            CreateTodo {
                session_id,
                title: "pre-existing".to_string(),
                description: None,
                priority: TodoPriority::Low,
                assigned_to_turn: None,
                tags: vec![],
            },
            CreateTodo {
                session_id,
                title: "new-item".to_string(),
                description: None,
                priority: TodoPriority::Medium,
                assigned_to_turn: None,
                tags: vec![],
            },
        ])
        .expect("batch");

    assert_eq!(items.len(), 2);
    assert_eq!(items[0].id, existing.id);
    assert_ne!(items[1].id, existing.id);
    assert_eq!(repo.list_all(session_id).expect("list").len(), 2);
}

#[test]
fn create_batch_empty_input_returns_empty() {
    let repo = repo();
    let session_id = Uuid::new_v4();

    let items = repo.create_batch(vec![]).expect("batch");
    assert!(items.is_empty());
    assert_eq!(repo.list_all(session_id).expect("list").len(), 0);
}

#[tokio::test]
async fn todo_create_batch_tool_creates_multiple_items() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("todos.sqlite");
    let session_id = Uuid::new_v4();
    let batch_tool = TodoCreateBatchTool::new(db_path.clone(), session_id);
    let query_tool = TodoQueryTool::new(db_path, session_id);

    let result = batch_tool
        .execute(serde_json::json!({
            "items": [
                {"title": "first", "priority": "high"},
                {"title": "second", "priority": "low"},
                {"title": "third"}
            ]
        }))
        .await;

    assert!(!result.is_error, "{}", result.content);
    assert!(result.content.contains("Created 3 todo(s)"));

    let queried = query_tool.execute(serde_json::json!({})).await;
    assert!(queried.content.contains("first"));
    assert!(queried.content.contains("second"));
    assert!(queried.content.contains("third"));
}

#[tokio::test]
async fn todo_create_batch_tool_rejects_empty_input() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("todos.sqlite");
    let session_id = Uuid::new_v4();
    let batch_tool = TodoCreateBatchTool::new(db_path, session_id);

    let result = batch_tool.execute(serde_json::json!({"items": []})).await;

    assert!(result.is_error);
    assert!(result.content.contains("at least one item"));
}

#[tokio::test]
async fn todo_create_batch_tool_idempotent_within_batch() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("todos.sqlite");
    let session_id = Uuid::new_v4();
    let batch_tool = TodoCreateBatchTool::new(db_path.clone(), session_id);
    let query_tool = TodoQueryTool::new(db_path, session_id);

    let result = batch_tool
        .execute(serde_json::json!({
            "items": [
                {"title": "dup", "priority": "high"},
                {"title": "dup", "priority": "low"}
            ]
        }))
        .await;

    assert!(!result.is_error, "{}", result.content);
    // Only 1 item despite 2 inputs (same title dedup)
    let queried = query_tool.execute(serde_json::json!({})).await;
    assert!(queried.content.contains("1 todo(s)"));
}

#[tokio::test]
async fn todo_update_batch_tool_updates_multiple_items() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("todos.sqlite");
    let session_id = Uuid::new_v4();
    let create_tool = TodoCreateTool::new(db_path.clone(), session_id);
    let batch_update_tool = TodoUpdateBatchTool::new(db_path.clone(), session_id);
    let query_tool = TodoQueryTool::new(db_path, session_id);

    let r1 = create_tool
        .execute(serde_json::json!({"title": "item-a", "priority": "low"}))
        .await;
    let id_a = extract_uuid_from_text(&r1.content);
    let r2 = create_tool
        .execute(serde_json::json!({"title": "item-b", "priority": "low"}))
        .await;
    let id_b = extract_uuid_from_text(&r2.content);

    let result = batch_update_tool
        .execute(serde_json::json!({
            "items": [
                {"id": id_a, "priority": "high"},
                {"id": id_b, "title": "renamed-b", "priority": "critical"}
            ]
        }))
        .await;

    assert!(!result.is_error, "{}", result.content);
    assert!(result.content.contains("Updated 2 todo(s)"));

    let queried = query_tool.execute(serde_json::json!({})).await;
    assert!(queried.content.contains("(high)"));
    assert!(queried.content.contains("renamed-b"));
    assert!(queried.content.contains("(critical)"));
}

#[tokio::test]
async fn todo_update_batch_tool_rejects_empty_input() {
    let dir = tempdir().expect("temp dir");
    let db_path = dir.path().join("todos.sqlite");
    let session_id = Uuid::new_v4();
    let tool = TodoUpdateBatchTool::new(db_path, session_id);

    let result = tool.execute(serde_json::json!({"items": []})).await;
    assert!(result.is_error);
    assert!(result.content.contains("at least one item"));
}
