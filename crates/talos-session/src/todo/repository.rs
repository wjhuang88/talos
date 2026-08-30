//! SQLite persistence for session todo state.

use chrono::{DateTime, Utc};
use rusqlite::{
    Connection, OptionalExtension, Result as RusqliteResult, Transaction, TransactionBehavior,
    params,
};
use serde_json;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::model::{
    CreateTodo, TodoDependency, TodoError, TodoItem, TodoPriority, TodoQuery, TodoStatus,
    TodoUpdate,
};
use talos_core::work::{WorkEdge, WorkEdgeIdentity, WorkGraph, WorkNode, validate_edge};

const TODO_SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Clone)]
struct StoredTodo {
    item: TodoItem,
    revision: i64,
}

#[derive(Debug, Clone)]
struct StoredEdge {
    dependency: TodoDependency,
    id: Uuid,
    revision: i64,
}

/// SQLite repository for session todo state.
#[derive(Debug)]
pub struct TodoRepository {
    conn: Connection,
    db_path: PathBuf,
}

impl TodoRepository {
    /// Open or create a todo database at the given path.
    ///
    /// # Errors
    ///
    /// Returns an error when the database file cannot be opened.
    pub fn new(path: &Path) -> Result<Self, TodoError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| TodoError::Database(err.to_string()))?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=5000;",
        )?;
        Ok(Self {
            conn,
            db_path: path.to_path_buf(),
        })
    }

    /// Return the path to the SQLite database.
    #[must_use]
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Initialize todo tables.
    ///
    /// # Errors
    ///
    /// Returns an error when SQLite rejects the schema.
    pub fn init_schema(&self) -> Result<(), TodoError> {
        let has_items = table_exists(&self.conn, "todo_items")?;
        let has_edges = table_exists(&self.conn, "todo_dependencies")?;
        if (object_exists(&self.conn, "todo_items")? && !has_items)
            || (object_exists(&self.conn, "todo_dependencies")? && !has_edges)
        {
            return Err(TodoError::Migration(
                "Todo schema object is not a table; refusing to alter it".to_string(),
            ));
        }
        if !has_items && !has_edges {
            self.conn.execute_batch(
                r#"
            CREATE TABLE IF NOT EXISTS todo_items (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL,
                priority TEXT NOT NULL,
                created_at TEXT NOT NULL,
                completed_at TEXT,
                assigned_to_turn TEXT,
                tags_json TEXT NOT NULL DEFAULT '[]',
                revision INTEGER NOT NULL DEFAULT 1
            );

            CREATE INDEX IF NOT EXISTS idx_todo_items_session_status
                ON todo_items(session_id, status);

            CREATE TABLE IF NOT EXISTS todo_dependencies (
                session_id TEXT NOT NULL,
                parent_id TEXT NOT NULL,
                child_id TEXT NOT NULL,
                edge_id TEXT NOT NULL,
                revision INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY (session_id, parent_id, child_id)
            );

            CREATE TABLE IF NOT EXISTS todo_dependency_history (
                edge_id TEXT PRIMARY KEY,
                revision INTEGER NOT NULL DEFAULT 1
            );
            "#,
            )?;
        }
        migrate_and_validate_schema(&self.conn, &self.db_path)?;
        ensure_edge_history_table(&self.conn)?;
        Ok(())
    }

    /// Create a todo item, or return an existing item with the same title
    /// in the same session (idempotent create).
    ///
    /// # Errors
    ///
    /// Returns an error when the item cannot be persisted or looked up.
    pub fn create(&self, input: CreateTodo) -> Result<TodoItem, TodoError> {
        self.create_batch(vec![input])?
            .pop()
            .ok_or_else(|| TodoError::Database("create returned no item".to_string()))
    }

    /// Create multiple todo items idempotently in one call.
    ///
    /// Each item follows the same idempotency rule as [`create`]: if an item
    /// with the same title already exists in the session, the existing item is
    /// returned unchanged. Items within the same batch that share a title also
    /// deduplicate to the first occurrence.
    ///
    /// # Errors
    ///
    /// Returns an error when any item cannot be persisted or looked up.
    pub fn create_batch(&self, inputs: Vec<CreateTodo>) -> Result<Vec<TodoItem>, TodoError> {
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        let mut results = Vec::with_capacity(inputs.len());
        for input in inputs {
            let existing = tx
                .query_row(
                    "SELECT id, session_id, title, description, status, priority, created_at, \
                     completed_at, assigned_to_turn, tags_json, revision FROM todo_items \
                     WHERE session_id = ?1 AND title = ?2",
                    params![input.session_id.to_string(), input.title.as_str()],
                    map_stored_todo,
                )
                .optional()?;
            if let Some(existing) = existing {
                results.push(existing.item);
                continue;
            }
            let item = TodoItem {
                id: Uuid::new_v4(),
                session_id: input.session_id,
                title: input.title,
                description: input.description,
                status: TodoStatus::Todo,
                priority: input.priority,
                created_at: Utc::now(),
                completed_at: None,
                assigned_to_turn: input.assigned_to_turn,
                tags: normalize_tags(input.tags),
            };
            let stored = StoredTodo {
                item: item.clone(),
                revision: 1,
            };
            tx.execute(
                "INSERT INTO todo_items (id, session_id, title, description, status, priority, \
                 created_at, completed_at, assigned_to_turn, tags_json, revision) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params_for_item(&stored)?,
            )?;
            results.push(item);
        }
        tx.commit()?;
        Ok(results)
    }

    /// Get one todo item by id within a session.
    ///
    /// # Errors
    ///
    /// Returns an error when SQLite fails.
    pub fn get(&self, session_id: Uuid, id: Uuid) -> Result<Option<TodoItem>, TodoError> {
        Ok(self.get_stored(session_id, id)?.map(|stored| stored.item))
    }

    fn get_stored(&self, session_id: Uuid, id: Uuid) -> Result<Option<StoredTodo>, TodoError> {
        self.conn
            .query_row(
                r#"
                SELECT id, session_id, title, description, status, priority, created_at,
                       completed_at, assigned_to_turn, tags_json, revision
                FROM todo_items
                WHERE session_id = ?1 AND id = ?2
                "#,
                params![session_id.to_string(), id.to_string()],
                map_stored_todo,
            )
            .optional()
            .map_err(TodoError::from)
    }

    /// List todo items for a session.
    ///
    /// # Errors
    ///
    /// Returns an error when SQLite fails or stored metadata cannot be parsed.
    pub fn list(&self, session_id: Uuid, query: TodoQuery) -> Result<Vec<TodoItem>, TodoError> {
        let mut items = self.list_all(session_id)?;
        if let Some(status) = query.status {
            items.retain(|item| item.status == status);
        }
        if let Some(priority) = query.priority {
            items.retain(|item| item.priority == priority);
        }
        if let Some(tag) = query.tag {
            items.retain(|item| item.tags.iter().any(|candidate| candidate == &tag));
        }
        Ok(items)
    }

    /// Update mutable todo fields.
    ///
    /// # Errors
    ///
    /// Returns [`TodoError::NotFound`] when the item does not exist in the session.
    pub fn update(
        &self,
        session_id: Uuid,
        id: Uuid,
        update: TodoUpdate,
    ) -> Result<TodoItem, TodoError> {
        let mut stored = self
            .get_stored(session_id, id)?
            .ok_or(TodoError::NotFound(id))?;
        let mut item = stored.item;
        if let Some(title) = update.title {
            item.title = title;
        }
        if let Some(description) = update.description {
            item.description = description;
        }
        if let Some(priority) = update.priority {
            item.priority = priority;
        }
        if let Some(assigned_to_turn) = update.assigned_to_turn {
            item.assigned_to_turn = assigned_to_turn;
        }
        if let Some(tags) = update.tags {
            item.tags = normalize_tags(tags);
        }
        stored.revision = next_revision(stored.revision, item.id)?;
        stored.item = item.clone();
        self.replace_item(&stored)?;
        Ok(item)
    }

    /// Update item status and maintain `completed_at`.
    ///
    /// # Errors
    ///
    /// Returns [`TodoError::NotFound`] when the item does not exist in the session.
    pub fn update_status(
        &self,
        session_id: Uuid,
        id: Uuid,
        status: TodoStatus,
    ) -> Result<TodoItem, TodoError> {
        let mut stored = self
            .get_stored(session_id, id)?
            .ok_or(TodoError::NotFound(id))?;
        let mut item = stored.item;
        item.status = status;
        item.completed_at = if status == TodoStatus::Completed {
            Some(Utc::now())
        } else {
            None
        };
        stored.revision = next_revision(stored.revision, item.id)?;
        stored.item = item.clone();
        self.replace_item(&stored)?;
        Ok(item)
    }

    /// Atomically update multiple Todo items. Any missing item, invalid revision or database
    /// error rolls the complete batch back.
    pub fn update_batch(
        &self,
        session_id: Uuid,
        updates: Vec<(Uuid, TodoUpdate)>,
    ) -> Result<Vec<TodoItem>, TodoError> {
        let tx = self.conn.unchecked_transaction()?;
        let mut updated = Vec::with_capacity(updates.len());
        for (id, update) in updates {
            let mut stored = tx
                .query_row(
                    "SELECT id, session_id, title, description, status, priority, created_at, \
                     completed_at, assigned_to_turn, tags_json, revision FROM todo_items \
                     WHERE session_id = ?1 AND id = ?2",
                    params![session_id.to_string(), id.to_string()],
                    map_stored_todo,
                )
                .optional()?
                .ok_or(TodoError::NotFound(id))?;
            apply_update(&mut stored.item, update);
            stored.revision = next_revision(stored.revision, id)?;
            let changed = tx.execute(
                "UPDATE todo_items SET title=?3, description=?4, status=?5, priority=?6, \
                 created_at=?7, completed_at=?8, assigned_to_turn=?9, tags_json=?10, revision=?11 \
                 WHERE id=?1 AND session_id=?2 AND revision=?12",
                params_for_item_with_prior_revision(&stored)?,
            )?;
            if changed != 1 {
                return Err(TodoError::RevisionConflict(id));
            }
            updated.push(stored.item);
        }
        tx.commit()?;
        Ok(updated)
    }

    /// Delete an item and any dependency edges that reference it.
    ///
    /// # Errors
    ///
    /// Returns an error when SQLite fails.
    pub fn delete(&mut self, session_id: Uuid, id: Uuid) -> Result<bool, TodoError> {
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        let affected_children = {
            let mut stmt = tx.prepare(
                "SELECT DISTINCT child_id FROM todo_dependencies \
                 WHERE session_id = ?1 AND parent_id = ?2",
            )?;
            stmt.query_map(params![session_id.to_string(), id.to_string()], |row| {
                parse_uuid_column(row.get::<_, String>(0)?, 0)
            })?
            .collect::<RusqliteResult<Vec<_>>>()?
        };
        tx.execute(
            "DELETE FROM todo_dependencies WHERE session_id = ?1 AND (parent_id = ?2 OR child_id = ?2)",
            params![session_id.to_string(), id.to_string()],
        )?;
        let deleted = tx.execute(
            "DELETE FROM todo_items WHERE session_id = ?1 AND id = ?2",
            params![session_id.to_string(), id.to_string()],
        )?;
        if deleted == 1 {
            for child_id in affected_children {
                advance_revision_on(&tx, session_id, child_id)?;
            }
        }
        tx.commit()?;
        Ok(deleted > 0)
    }

    /// Add a dependency edge after validating item existence and acyclicity.
    ///
    /// # Errors
    ///
    /// Returns [`TodoError::DependencyCycle`] if adding the edge would create a cycle.
    pub fn add_dependency(
        &self,
        session_id: Uuid,
        parent_id: Uuid,
        child_id: Uuid,
    ) -> Result<TodoDependency, TodoError> {
        if parent_id == child_id {
            return Err(TodoError::SelfDependency(parent_id));
        }
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        require_item_on(&tx, session_id, parent_id)?;
        require_item_on(&tx, session_id, child_id)?;
        if path_exists_on(&tx, session_id, child_id, parent_id)? {
            return Err(TodoError::DependencyCycle {
                parent_id,
                child_id,
            });
        }
        let edge_id = stable_edge_id(session_id, parent_id, child_id);
        let prior_revision: Option<i64> = tx
            .query_row(
                "SELECT revision FROM todo_dependency_history WHERE edge_id = ?1",
                params![edge_id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        let edge_revision = prior_revision
            .map(|revision| next_revision(revision, edge_id))
            .transpose()?
            .unwrap_or(1);
        let inserted = tx.execute(
            "INSERT OR IGNORE INTO todo_dependencies \
             (session_id, parent_id, child_id, edge_id, revision) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                session_id.to_string(),
                parent_id.to_string(),
                child_id.to_string(),
                edge_id.to_string(),
                edge_revision,
            ],
        )?;
        if inserted == 1 {
            tx.execute(
                "INSERT INTO todo_dependency_history (edge_id, revision) VALUES (?1, ?2) \
                 ON CONFLICT(edge_id) DO UPDATE SET revision=excluded.revision",
                params![edge_id.to_string(), edge_revision],
            )?;
            advance_revision_on(&tx, session_id, child_id)?;
        }
        tx.commit()?;
        Ok(TodoDependency {
            session_id,
            parent_id,
            child_id,
        })
    }

    fn replace_item(&self, stored: &StoredTodo) -> Result<(), TodoError> {
        let changed = self.conn.execute(
            r#"
            UPDATE todo_items
            SET title = ?3,
                description = ?4,
                status = ?5,
                priority = ?6,
                created_at = ?7,
                completed_at = ?8,
                assigned_to_turn = ?9,
                tags_json = ?10,
                revision = ?11
            WHERE id = ?1 AND session_id = ?2 AND revision = ?12
            "#,
            params_for_item_with_prior_revision(stored)?,
        )?;
        if changed == 1 {
            Ok(())
        } else {
            Err(TodoError::RevisionConflict(stored.item.id))
        }
    }

    /// Remove a dependency edge.
    ///
    /// # Errors
    ///
    /// Returns an error when SQLite fails.
    pub fn remove_dependency(
        &self,
        session_id: Uuid,
        parent_id: Uuid,
        child_id: Uuid,
    ) -> Result<bool, TodoError> {
        let tx = Transaction::new_unchecked(&self.conn, TransactionBehavior::Immediate)?;
        let deleted = tx.execute(
            "DELETE FROM todo_dependencies WHERE session_id = ?1 AND parent_id = ?2 AND child_id = ?3",
            params![
                session_id.to_string(),
                parent_id.to_string(),
                child_id.to_string(),
            ],
        )?;
        if deleted == 1 {
            advance_revision_on(&tx, session_id, child_id)?;
        }
        tx.commit()?;
        Ok(deleted > 0)
    }

    /// List all dependency edges for a session.
    ///
    /// # Errors
    ///
    /// Returns an error when SQLite fails.
    pub fn list_dependencies(&self, session_id: Uuid) -> Result<Vec<TodoDependency>, TodoError> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, parent_id, child_id FROM todo_dependencies \
             WHERE session_id = ?1 ORDER BY parent_id ASC, child_id ASC",
        )?;
        let deps = stmt
            .query_map(params![session_id.to_string()], |row| {
                Ok(TodoDependency {
                    session_id: parse_uuid_column(row.get::<_, String>(0)?, 0)?,
                    parent_id: parse_uuid_column(row.get::<_, String>(1)?, 1)?,
                    child_id: parse_uuid_column(row.get::<_, String>(2)?, 2)?,
                })
            })?
            .collect::<RusqliteResult<Vec<_>>>()?;
        Ok(deps)
    }

    /// Project the session's Todo records into canonical WorkUnit values.
    ///
    /// Todo remains the sole durable authority during the compatibility window; this is a
    /// read-only projection and never creates or mutates another repository.
    pub fn list_work_units(&self, session_id: Uuid) -> Result<Vec<WorkNode>, TodoError> {
        Ok(self
            .list_all_stored(session_id)?
            .iter()
            .map(|stored| stored.item.as_work_unit(stored.revision as u64))
            .collect())
    }

    /// Project the session's Todo dependency edges into canonical work edges.
    pub fn list_work_edges(&self, session_id: Uuid) -> Result<Vec<WorkEdge>, TodoError> {
        Ok(self
            .list_stored_edges(session_id)?
            .into_iter()
            .map(|stored| WorkEdge {
                identity: WorkEdgeIdentity {
                    id: stored.id,
                    revision: stored.revision as u64,
                },
                parent_id: stored.dependency.parent_id,
                child_id: stored.dependency.child_id,
            })
            .collect())
    }

    /// Load one validated canonical graph snapshot from the Todo authority.
    ///
    /// Both projections are read through one SQLite transaction so callers cannot observe a
    /// partially changed node/edge set. Any orphan, duplicate or cyclic legacy data fails closed.
    pub fn load_work_graph(&self, session_id: Uuid) -> Result<WorkGraph, TodoError> {
        let tx = self.conn.unchecked_transaction()?;
        let nodes = {
            let mut stmt = tx.prepare(
                "SELECT id, session_id, title, description, status, priority, created_at, \
                 completed_at, assigned_to_turn, tags_json, revision FROM todo_items \
                 WHERE session_id = ?1 ORDER BY created_at ASC, id ASC",
            )?;
            stmt.query_map(params![session_id.to_string()], map_stored_todo)?
                .collect::<RusqliteResult<Vec<_>>>()?
                .into_iter()
                .map(|stored| stored.item.as_work_unit(stored.revision as u64))
                .collect::<Vec<_>>()
        };
        let edges = {
            let mut stmt = tx.prepare(
                "SELECT session_id, parent_id, child_id, edge_id, revision \
                 FROM todo_dependencies WHERE session_id = ?1 \
                 ORDER BY parent_id ASC, child_id ASC",
            )?;
            stmt.query_map(params![session_id.to_string()], |row| {
                Ok(WorkEdge {
                    identity: WorkEdgeIdentity {
                        id: parse_uuid_column(row.get::<_, String>(3)?, 3)?,
                        revision: parse_revision_column(row.get::<_, i64>(4)?, 4)?,
                    },
                    parent_id: parse_uuid_column(row.get::<_, String>(1)?, 1)?,
                    child_id: parse_uuid_column(row.get::<_, String>(2)?, 2)?,
                })
            })?
            .collect::<RusqliteResult<Vec<_>>>()?
        };
        let graph = WorkGraph::new(nodes, edges)
            .map_err(|error| TodoError::Migration(format!("invalid canonical graph: {error}")))?;
        tx.commit()?;
        Ok(graph)
    }

    fn list_stored_edges(&self, session_id: Uuid) -> Result<Vec<StoredEdge>, TodoError> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, parent_id, child_id, edge_id, revision \
             FROM todo_dependencies WHERE session_id = ?1 \
             ORDER BY parent_id ASC, child_id ASC",
        )?;
        let edges = stmt
            .query_map(params![session_id.to_string()], |row| {
                Ok(StoredEdge {
                    dependency: TodoDependency {
                        session_id: parse_uuid_column(row.get::<_, String>(0)?, 0)?,
                        parent_id: parse_uuid_column(row.get::<_, String>(1)?, 1)?,
                        child_id: parse_uuid_column(row.get::<_, String>(2)?, 2)?,
                    },
                    id: parse_uuid_column(row.get::<_, String>(3)?, 3)?,
                    revision: parse_revision_column(row.get::<_, i64>(4)?, 4)? as i64,
                })
            })?
            .collect::<RusqliteResult<Vec<_>>>()?;
        Ok(edges)
    }

    pub(super) fn list_all(&self, session_id: Uuid) -> Result<Vec<TodoItem>, TodoError> {
        Ok(self
            .list_all_stored(session_id)?
            .into_iter()
            .map(|stored| stored.item)
            .collect())
    }

    fn list_all_stored(&self, session_id: Uuid) -> Result<Vec<StoredTodo>, TodoError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, session_id, title, description, status, priority, created_at,
                   completed_at, assigned_to_turn, tags_json, revision
            FROM todo_items
            WHERE session_id = ?1
            ORDER BY created_at ASC, id ASC
            "#,
        )?;
        let items = stmt
            .query_map(params![session_id.to_string()], map_stored_todo)?
            .collect::<RusqliteResult<Vec<_>>>()?;
        Ok(items)
    }
}

fn require_item_on(conn: &Connection, session_id: Uuid, id: Uuid) -> Result<(), TodoError> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM todo_items WHERE session_id = ?1 AND id = ?2)",
        params![session_id.to_string(), id.to_string()],
        |row| row.get(0),
    )?;
    if exists {
        Ok(())
    } else {
        Err(TodoError::NotFound(id))
    }
}

fn path_exists_on(
    conn: &Connection,
    session_id: Uuid,
    from: Uuid,
    to: Uuid,
) -> Result<bool, TodoError> {
    let mut stmt =
        conn.prepare("SELECT parent_id, child_id FROM todo_dependencies WHERE session_id = ?1")?;
    let deps = stmt
        .query_map(params![session_id.to_string()], |row| {
            Ok((
                parse_uuid_column(row.get::<_, String>(0)?, 0)?,
                parse_uuid_column(row.get::<_, String>(1)?, 1)?,
            ))
        })?
        .collect::<RusqliteResult<Vec<_>>>()?;
    let mut graph: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for (parent_id, child_id) in deps {
        graph.entry(parent_id).or_default().push(child_id);
    }
    let mut stack = vec![from];
    let mut seen = HashSet::new();
    while let Some(node) = stack.pop() {
        if node == to {
            return Ok(true);
        }
        if !seen.insert(node) {
            continue;
        }
        if let Some(children) = graph.get(&node) {
            stack.extend(children.iter().copied());
        }
    }
    Ok(false)
}

fn advance_revision_on(conn: &Connection, session_id: Uuid, id: Uuid) -> Result<(), TodoError> {
    let changed = conn.execute(
        "UPDATE todo_items SET revision = revision + 1 \
         WHERE session_id = ?1 AND id = ?2 AND revision < ?3",
        params![session_id.to_string(), id.to_string(), i64::MAX],
    )?;
    if changed == 1 {
        Ok(())
    } else {
        Err(TodoError::RevisionExhausted(id))
    }
}

fn params_for_item(stored: &StoredTodo) -> Result<[rusqlite::types::Value; 11], TodoError> {
    let item = &stored.item;
    Ok([
        item.id.to_string().into(),
        item.session_id.to_string().into(),
        item.title.clone().into(),
        item.description.clone().unwrap_or_default().into(),
        item.status.as_str().to_string().into(),
        item.priority.as_str().to_string().into(),
        item.created_at.to_rfc3339().into(),
        item.completed_at
            .map(|completed_at| completed_at.to_rfc3339())
            .unwrap_or_default()
            .into(),
        item.assigned_to_turn.clone().unwrap_or_default().into(),
        serde_json::to_string(&item.tags)?.into(),
        stored.revision.into(),
    ])
}

fn params_for_item_with_prior_revision(
    stored: &StoredTodo,
) -> Result<[rusqlite::types::Value; 12], TodoError> {
    let values = params_for_item(stored)?;
    let prior_revision = stored
        .revision
        .checked_sub(1)
        .ok_or(TodoError::RevisionExhausted(stored.item.id))?;
    Ok([
        values[0].clone(),
        values[1].clone(),
        values[2].clone(),
        values[3].clone(),
        values[4].clone(),
        values[5].clone(),
        values[6].clone(),
        values[7].clone(),
        values[8].clone(),
        values[9].clone(),
        values[10].clone(),
        prior_revision.into(),
    ])
}

fn map_stored_todo(row: &rusqlite::Row<'_>) -> RusqliteResult<StoredTodo> {
    let id = parse_uuid_column(row.get::<_, String>(0)?, 0)?;
    let session_id = parse_uuid_column(row.get::<_, String>(1)?, 1)?;
    let created_at = parse_datetime_column(row.get::<_, String>(6)?, 6)?;
    let completed_at = match row.get::<_, Option<String>>(7)? {
        None => None,
        Some(value) if value.is_empty() => None,
        Some(value) => Some(parse_datetime_column(value, 7)?),
    };
    let tags_json: String = row.get(9)?;
    let tags = serde_json::from_str::<Vec<String>>(&tags_json).map_err(|_| {
        rusqlite::Error::InvalidColumnType(9, tags_json, rusqlite::types::Type::Text)
    })?;
    let description = row
        .get::<_, Option<String>>(3)?
        .and_then(empty_string_to_none);
    let assigned_to_turn = row
        .get::<_, Option<String>>(8)?
        .and_then(empty_string_to_none);
    let status_value = row.get::<_, String>(4)?;
    let status = TodoStatus::from_str(&status_value).ok_or_else(|| {
        rusqlite::Error::InvalidColumnType(4, status_value, rusqlite::types::Type::Text)
    })?;
    let priority_value = row.get::<_, String>(5)?;
    let priority = TodoPriority::from_str(&priority_value).ok_or_else(|| {
        rusqlite::Error::InvalidColumnType(5, priority_value, rusqlite::types::Type::Text)
    })?;

    Ok(StoredTodo {
        item: TodoItem {
            id,
            session_id,
            title: row.get(2)?,
            description,
            status,
            priority,
            created_at,
            completed_at,
            assigned_to_turn,
            tags,
        },
        revision: parse_revision_column(row.get::<_, i64>(10)?, 10)? as i64,
    })
}

fn migrate_and_validate_schema(conn: &Connection, db_path: &Path) -> Result<(), TodoError> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version > TODO_SCHEMA_VERSION {
        return Err(TodoError::Migration(format!(
            "database version {version} is newer than supported version {TODO_SCHEMA_VERSION}"
        )));
    }

    let item_columns = table_columns(conn, "todo_items")?;
    let edge_columns = table_columns(conn, "todo_dependencies")?;
    let legacy_items = [
        "id",
        "session_id",
        "title",
        "description",
        "status",
        "priority",
        "created_at",
        "completed_at",
        "assigned_to_turn",
        "tags_json",
    ];
    let current_items = [
        "id",
        "session_id",
        "title",
        "description",
        "status",
        "priority",
        "created_at",
        "completed_at",
        "assigned_to_turn",
        "tags_json",
        "revision",
    ];
    let legacy_edges = ["session_id", "parent_id", "child_id"];
    let current_edges = ["session_id", "parent_id", "child_id", "edge_id", "revision"];
    let legacy = item_columns == legacy_items
        && edge_columns == legacy_edges
        && validate_schema_metadata(conn, false)?;
    let current = item_columns == current_items
        && edge_columns == current_edges
        && validate_schema_metadata(conn, true)?;
    if !legacy && !current {
        return Err(TodoError::Migration(
            "partial or unknown Todo schema; refusing a lossy migration".to_string(),
        ));
    }

    if legacy {
        validate_legacy_rows(conn)?;
        let backup_path = PathBuf::from(format!("{}.pre-work-v1.bak", db_path.display()));
        if backup_path.exists() {
            return Err(TodoError::Migration(format!(
                "migration backup already exists at {}; inspect or remove it before retrying",
                backup_path.display()
            )));
        }
        conn.backup("main", &backup_path, None)?;
        let tx = Transaction::new_unchecked(conn, TransactionBehavior::Immediate)?;
        tx.execute(
            "ALTER TABLE todo_items ADD COLUMN revision INTEGER NOT NULL DEFAULT 1",
            [],
        )?;
        tx.execute("ALTER TABLE todo_dependencies ADD COLUMN edge_id TEXT", [])?;
        tx.execute(
            "ALTER TABLE todo_dependencies ADD COLUMN revision INTEGER NOT NULL DEFAULT 1",
            [],
        )?;
        let legacy_edges = {
            let mut stmt = tx.prepare(
                "SELECT session_id, parent_id, child_id FROM todo_dependencies \
                 ORDER BY session_id, parent_id, child_id",
            )?;
            stmt.query_map([], |row| {
                Ok((
                    parse_uuid_column(row.get::<_, String>(0)?, 0)?,
                    parse_uuid_column(row.get::<_, String>(1)?, 1)?,
                    parse_uuid_column(row.get::<_, String>(2)?, 2)?,
                ))
            })?
            .collect::<RusqliteResult<Vec<_>>>()?
        };
        for (session_id, parent_id, child_id) in legacy_edges {
            tx.execute(
                "UPDATE todo_dependencies SET edge_id = ?4 \
                 WHERE session_id = ?1 AND parent_id = ?2 AND child_id = ?3",
                params![
                    session_id.to_string(),
                    parent_id.to_string(),
                    child_id.to_string(),
                    stable_edge_id(session_id, parent_id, child_id).to_string(),
                ],
            )?;
        }
        tx.execute_batch(
            r#"
            CREATE TABLE todo_dependencies__work_v1 (
                session_id TEXT NOT NULL,
                parent_id TEXT NOT NULL,
                child_id TEXT NOT NULL,
                edge_id TEXT NOT NULL,
                revision INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY (session_id, parent_id, child_id)
            );
            INSERT INTO todo_dependencies__work_v1
                (session_id, parent_id, child_id, edge_id, revision)
                SELECT session_id, parent_id, child_id, edge_id, revision
                FROM todo_dependencies;
            DROP TABLE todo_dependencies;
            ALTER TABLE todo_dependencies__work_v1 RENAME TO todo_dependencies;
            "#,
        )?;
        tx.pragma_update(None, "user_version", TODO_SCHEMA_VERSION)?;
        validate_current_rows(&tx)?;
        tx.commit()?;
    } else {
        validate_current_rows(conn)?;
        if version == 0 {
            conn.pragma_update(None, "user_version", TODO_SCHEMA_VERSION)?;
        }
    }
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_todo_items_session_title \
         ON todo_items(session_id, title)",
        [],
    )?;
    Ok(())
}

fn table_columns(conn: &Connection, table: &str) -> Result<Vec<String>, TodoError> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    Ok(stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<RusqliteResult<Vec<_>>>()?)
}

fn validate_schema_metadata(conn: &Connection, current: bool) -> Result<bool, TodoError> {
    let mut stmt = conn.prepare("PRAGMA table_info(todo_items)")?;
    let item_meta = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })?
        .collect::<RusqliteResult<Vec<_>>>()?;
    let expected_item_pk = item_meta.iter().any(|row| row.0 == "id" && row.4 == 1)
        && item_meta
            .iter()
            .filter(|row| row.0 != "id")
            .all(|row| row.4 == 0);
    if !expected_item_pk {
        return Ok(false);
    }
    if current {
        let revision = item_meta.iter().find(|row| row.0 == "revision");
        if !matches!(revision, Some((_, ty, notnull, default, _)) if ty.eq_ignore_ascii_case("INTEGER") && *notnull == 1 && default.as_deref() == Some("1"))
        {
            return Ok(false);
        }
    }
    let mut edge_stmt = conn.prepare("PRAGMA table_info(todo_dependencies)")?;
    let edge_meta = edge_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })?
        .collect::<RusqliteResult<Vec<_>>>()?;
    let edge_pk = ["session_id", "parent_id", "child_id"]
        .iter()
        .enumerate()
        .all(|(position, name)| {
            edge_meta
                .iter()
                .any(|row| row.0 == *name && row.4 == (position as i64 + 1))
        })
        && edge_meta
            .iter()
            .filter(|row| !["session_id", "parent_id", "child_id"].contains(&row.0.as_str()))
            .all(|row| row.4 == 0);
    if !edge_pk {
        return Ok(false);
    }
    if current {
        for name in ["edge_id", "revision"] {
            let Some((_, ty, notnull, default, _)) = edge_meta.iter().find(|row| row.0 == name)
            else {
                return Ok(false);
            };
            if *notnull != 1 || default.as_deref() != Some("1") && name == "revision" {
                return Ok(false);
            }
            if name == "edge_id" && !ty.eq_ignore_ascii_case("TEXT") {
                return Ok(false);
            }
            if name == "revision" && !ty.eq_ignore_ascii_case("INTEGER") {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

fn validate_legacy_rows(conn: &Connection) -> Result<(), TodoError> {
    validate_item_rows(conn, false)?;
    validate_edge_rows(conn, false)
}

fn validate_current_rows(conn: &Connection) -> Result<(), TodoError> {
    validate_item_rows(conn, true)?;
    validate_edge_rows(conn, true)
}

fn validate_item_rows(conn: &Connection, with_revision: bool) -> Result<(), TodoError> {
    let revision = if with_revision { ", revision" } else { "" };
    let mut stmt = conn.prepare(&format!(
        "SELECT id, session_id, title, description, status, priority, created_at, \
         completed_at, assigned_to_turn, tags_json{revision} FROM todo_items"
    ))?;
    let mut rows = stmt.query([])?;
    let mut identities = HashSet::new();
    let mut titles = HashSet::new();
    while let Some(row) = rows.next()? {
        let id = parse_uuid_column(row.get::<_, String>(0)?, 0)?;
        let session_id = parse_uuid_column(row.get::<_, String>(1)?, 1)?;
        if !identities.insert((session_id, id)) {
            return Err(TodoError::Migration("duplicate Todo identity".to_string()));
        }
        let title = row.get::<_, String>(2)?;
        if !titles.insert((session_id, title)) {
            return Err(TodoError::Migration(
                "duplicate Todo title in session".to_string(),
            ));
        }
        let status = row.get::<_, String>(4)?;
        let priority = row.get::<_, String>(5)?;
        if TodoStatus::from_str(&status).is_none() || TodoPriority::from_str(&priority).is_none() {
            return Err(TodoError::Migration(
                "unknown Todo status or priority".to_string(),
            ));
        }
        parse_datetime_column(row.get::<_, String>(6)?, 6)?;
        if let Some(value) = row.get::<_, Option<String>>(7)?
            && !value.is_empty()
        {
            parse_datetime_column(value, 7)?;
        }
        let tags = row.get::<_, String>(9)?;
        serde_json::from_str::<Vec<String>>(&tags)
            .map_err(|_| TodoError::Migration("invalid Todo tags JSON".to_string()))?;
        if with_revision {
            let revision = parse_revision_column(row.get::<_, i64>(10)?, 10)?;
            if revision == 0 {
                return Err(TodoError::Migration(
                    "Todo revision must be positive".to_string(),
                ));
            }
        }
    }
    Ok(())
}

fn validate_edge_rows(conn: &Connection, with_identity: bool) -> Result<(), TodoError> {
    let suffix = if with_identity {
        ", edge_id, revision"
    } else {
        ""
    };
    let mut stmt = conn.prepare(&format!(
        "SELECT session_id, parent_id, child_id{suffix} FROM todo_dependencies \
         ORDER BY session_id, parent_id, child_id"
    ))?;
    let mut rows = stmt.query([])?;
    let mut per_session: HashMap<Uuid, Vec<WorkEdge>> = HashMap::new();
    let mut edge_ids = HashSet::new();
    while let Some(row) = rows.next()? {
        let session_id = parse_uuid_column(row.get::<_, String>(0)?, 0)?;
        let parent_id = parse_uuid_column(row.get::<_, String>(1)?, 1)?;
        let child_id = parse_uuid_column(row.get::<_, String>(2)?, 2)?;
        let (id, revision) = if with_identity {
            (
                parse_uuid_column(row.get::<_, String>(3)?, 3)?,
                parse_revision_column(row.get::<_, i64>(4)?, 4)?,
            )
        } else {
            (stable_edge_id(session_id, parent_id, child_id), 1)
        };
        if with_identity && id != stable_edge_id(session_id, parent_id, child_id) {
            return Err(TodoError::Migration(
                "Todo dependency edge identity mismatch".to_string(),
            ));
        }
        if !edge_ids.insert(id) {
            return Err(TodoError::Migration(
                "duplicate Todo dependency edge identity".to_string(),
            ));
        }
        if revision == 0 {
            return Err(TodoError::Migration(
                "Todo dependency revision must be positive".to_string(),
            ));
        }
        let item_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM todo_items WHERE session_id = ?1 AND id IN (?2, ?3)",
            params![
                session_id.to_string(),
                parent_id.to_string(),
                child_id.to_string()
            ],
            |candidate| candidate.get(0),
        )?;
        if item_count != 2 {
            return Err(TodoError::Migration("orphan Todo dependency".to_string()));
        }
        let edge = WorkEdge {
            identity: WorkEdgeIdentity { id, revision },
            parent_id,
            child_id,
        };
        let accepted = per_session.entry(session_id).or_default();
        validate_edge(accepted.iter().copied(), edge)
            .map_err(|err| TodoError::Migration(format!("invalid Todo dependency graph: {err}")))?;
        accepted.push(edge);
    }
    Ok(())
}

fn stable_edge_id(session_id: Uuid, parent_id: Uuid, child_id: Uuid) -> Uuid {
    let mut hasher = Sha256::new();
    hasher.update(session_id.as_bytes());
    hasher.update(parent_id.as_bytes());
    hasher.update(child_id.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    Uuid::from_bytes(bytes)
}

/// Return whether a named object is a real SQLite table.
///
/// Schema initialization must inspect existing metadata before issuing any DDL. In particular,
/// a database containing a view (or only one of the Todo tables) must be rejected rather than
/// having `CREATE TABLE IF NOT EXISTS` silently alter its shape.
fn table_exists(conn: &Connection, name: &str) -> Result<bool, TodoError> {
    Ok(object_type(conn, name)?.as_deref() == Some("table"))
}

fn object_exists(conn: &Connection, name: &str) -> Result<bool, TodoError> {
    Ok(object_type(conn, name)?.is_some())
}

fn object_type(conn: &Connection, name: &str) -> Result<Option<String>, TodoError> {
    let object_type = conn
        .query_row(
            "SELECT type FROM sqlite_master WHERE name = ?1",
            params![name],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    Ok(object_type)
}

fn ensure_edge_history_table(conn: &Connection) -> Result<(), TodoError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS todo_dependency_history (\
            edge_id TEXT PRIMARY KEY, revision INTEGER NOT NULL DEFAULT 1\
        );",
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO todo_dependency_history (edge_id, revision) \
         SELECT edge_id, revision FROM todo_dependencies",
        [],
    )?;
    Ok(())
}

fn next_revision(revision: i64, id: Uuid) -> Result<i64, TodoError> {
    revision
        .checked_add(1)
        .ok_or(TodoError::RevisionExhausted(id))
}

fn apply_update(item: &mut TodoItem, update: TodoUpdate) {
    if let Some(title) = update.title {
        item.title = title;
    }
    if let Some(description) = update.description {
        item.description = description;
    }
    if let Some(priority) = update.priority {
        item.priority = priority;
    }
    if let Some(assigned_to_turn) = update.assigned_to_turn {
        item.assigned_to_turn = assigned_to_turn;
    }
    if let Some(tags) = update.tags {
        item.tags = normalize_tags(tags);
    }
}

fn parse_uuid_column(value: String, column: usize) -> RusqliteResult<Uuid> {
    Uuid::parse_str(&value)
        .map_err(|_| rusqlite::Error::InvalidColumnType(column, value, rusqlite::types::Type::Text))
}

fn parse_revision_column(value: i64, column: usize) -> RusqliteResult<u64> {
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(column, value))
}

fn parse_datetime_column(value: String, column: usize) -> RusqliteResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| rusqlite::Error::InvalidColumnType(column, value, rusqlite::types::Type::Text))
}

fn empty_string_to_none(value: String) -> Option<String> {
    if value.is_empty() { None } else { Some(value) }
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut tags: Vec<String> = tags
        .into_iter()
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect();
    tags.sort();
    tags.dedup();
    tags
}
