//! SQLite persistence for session todo state.

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, Result as RusqliteResult, params};
use serde_json;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::model::{
    CreateTodo, TodoDependency, TodoError, TodoItem, TodoPriority, TodoQuery, TodoStatus,
    TodoUpdate,
};

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
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
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
                tags_json TEXT NOT NULL DEFAULT '[]'
            );

            CREATE INDEX IF NOT EXISTS idx_todo_items_session_status
                ON todo_items(session_id, status);

            CREATE TABLE IF NOT EXISTS todo_dependencies (
                session_id TEXT NOT NULL,
                parent_id TEXT NOT NULL,
                child_id TEXT NOT NULL,
                PRIMARY KEY (session_id, parent_id, child_id)
            );
            "#,
        )?;
        Ok(())
    }

    /// Create a todo item, or return an existing item with the same title
    /// in the same session (idempotent create).
    ///
    /// # Errors
    ///
    /// Returns an error when the item cannot be persisted or looked up.
    pub fn create(&self, input: CreateTodo) -> Result<TodoItem, TodoError> {
        // Idempotency: return existing item with same title in this session.
        if let Some(existing) = self.find_by_title(input.session_id, &input.title)? {
            return Ok(existing);
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
        self.insert_item(&item)?;
        Ok(item)
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
        let mut results = Vec::with_capacity(inputs.len());
        for input in inputs {
            results.push(self.create(input)?);
        }
        Ok(results)
    }

    /// Get one todo item by id within a session.
    ///
    /// # Errors
    ///
    /// Returns an error when SQLite fails.
    pub fn get(&self, session_id: Uuid, id: Uuid) -> Result<Option<TodoItem>, TodoError> {
        self.conn
            .query_row(
                r#"
                SELECT id, session_id, title, description, status, priority, created_at,
                       completed_at, assigned_to_turn, tags_json
                FROM todo_items
                WHERE session_id = ?1 AND id = ?2
                "#,
                params![session_id.to_string(), id.to_string()],
                map_todo_item,
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
        let mut item = self.get(session_id, id)?.ok_or(TodoError::NotFound(id))?;
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
        self.replace_item(&item)?;
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
        let mut item = self.get(session_id, id)?.ok_or(TodoError::NotFound(id))?;
        item.status = status;
        item.completed_at = if status == TodoStatus::Completed {
            Some(Utc::now())
        } else {
            None
        };
        self.replace_item(&item)?;
        Ok(item)
    }

    /// Delete an item and any dependency edges that reference it.
    ///
    /// # Errors
    ///
    /// Returns an error when SQLite fails.
    pub fn delete(&mut self, session_id: Uuid, id: Uuid) -> Result<bool, TodoError> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "DELETE FROM todo_dependencies WHERE session_id = ?1 AND (parent_id = ?2 OR child_id = ?2)",
            params![session_id.to_string(), id.to_string()],
        )?;
        let deleted = tx.execute(
            "DELETE FROM todo_items WHERE session_id = ?1 AND id = ?2",
            params![session_id.to_string(), id.to_string()],
        )?;
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
        self.require_item(session_id, parent_id)?;
        self.require_item(session_id, child_id)?;
        if self.path_exists(session_id, child_id, parent_id)? {
            return Err(TodoError::DependencyCycle {
                parent_id,
                child_id,
            });
        }
        self.conn.execute(
            r#"
            INSERT OR IGNORE INTO todo_dependencies (session_id, parent_id, child_id)
            VALUES (?1, ?2, ?3)
            "#,
            params![
                session_id.to_string(),
                parent_id.to_string(),
                child_id.to_string(),
            ],
        )?;
        Ok(TodoDependency {
            session_id,
            parent_id,
            child_id,
        })
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
        let deleted = self.conn.execute(
            "DELETE FROM todo_dependencies WHERE session_id = ?1 AND parent_id = ?2 AND child_id = ?3",
            params![
                session_id.to_string(),
                parent_id.to_string(),
                child_id.to_string(),
            ],
        )?;
        Ok(deleted > 0)
    }

    /// List all dependency edges for a session.
    ///
    /// # Errors
    ///
    /// Returns an error when SQLite fails.
    pub fn list_dependencies(&self, session_id: Uuid) -> Result<Vec<TodoDependency>, TodoError> {
        let mut stmt = self.conn.prepare(
            "SELECT session_id, parent_id, child_id FROM todo_dependencies WHERE session_id = ?1",
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

    fn insert_item(&self, item: &TodoItem) -> Result<(), TodoError> {
        self.conn.execute(
            r#"
            INSERT INTO todo_items (
                id, session_id, title, description, status, priority, created_at, completed_at,
                assigned_to_turn, tags_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params_for_item(item)?,
        )?;
        Ok(())
    }

    fn replace_item(&self, item: &TodoItem) -> Result<(), TodoError> {
        self.conn.execute(
            r#"
            UPDATE todo_items
            SET title = ?3,
                description = ?4,
                status = ?5,
                priority = ?6,
                created_at = ?7,
                completed_at = ?8,
                assigned_to_turn = ?9,
                tags_json = ?10
            WHERE id = ?1 AND session_id = ?2
            "#,
            params_for_item(item)?,
        )?;
        Ok(())
    }

    pub(super) fn list_all(&self, session_id: Uuid) -> Result<Vec<TodoItem>, TodoError> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, session_id, title, description, status, priority, created_at,
                   completed_at, assigned_to_turn, tags_json
            FROM todo_items
            WHERE session_id = ?1
            ORDER BY created_at ASC, id ASC
            "#,
        )?;
        let items = stmt
            .query_map(params![session_id.to_string()], map_todo_item)?
            .collect::<RusqliteResult<Vec<_>>>()?;
        Ok(items)
    }

    /// Find a todo item by session and exact title match (case-sensitive).
    /// Used for idempotent create — repeated `todo_create` for the same
    /// title in the same session returns the existing item.
    fn find_by_title(&self, session_id: Uuid, title: &str) -> Result<Option<TodoItem>, TodoError> {
        self.conn
            .query_row(
                r#"
                SELECT id, session_id, title, description, status, priority, created_at,
                       completed_at, assigned_to_turn, tags_json
                FROM todo_items
                WHERE session_id = ?1 AND title = ?2
                "#,
                params![session_id.to_string(), title],
                map_todo_item,
            )
            .optional()
            .map_err(TodoError::from)
    }

    fn require_item(&self, session_id: Uuid, id: Uuid) -> Result<(), TodoError> {
        if self.get(session_id, id)?.is_some() {
            Ok(())
        } else {
            Err(TodoError::NotFound(id))
        }
    }

    fn path_exists(&self, session_id: Uuid, from: Uuid, to: Uuid) -> Result<bool, TodoError> {
        let deps = self.list_dependencies(session_id)?;
        let mut graph: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for dep in deps {
            graph.entry(dep.parent_id).or_default().push(dep.child_id);
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
}

fn params_for_item(item: &TodoItem) -> Result<[String; 10], TodoError> {
    Ok([
        item.id.to_string(),
        item.session_id.to_string(),
        item.title.clone(),
        item.description.clone().unwrap_or_default(),
        item.status.as_str().to_string(),
        item.priority.as_str().to_string(),
        item.created_at.to_rfc3339(),
        item.completed_at
            .map(|completed_at| completed_at.to_rfc3339())
            .unwrap_or_default(),
        item.assigned_to_turn.clone().unwrap_or_default(),
        serde_json::to_string(&item.tags)?,
    ])
}

fn map_todo_item(row: &rusqlite::Row<'_>) -> RusqliteResult<TodoItem> {
    let id = parse_uuid_column(row.get::<_, String>(0)?, 0)?;
    let session_id = parse_uuid_column(row.get::<_, String>(1)?, 1)?;
    let created_at = parse_datetime_column(row.get::<_, String>(6)?, 6)?;
    let completed_at = match row.get::<_, String>(7)?.as_str() {
        "" => None,
        value => Some(parse_datetime_column(value.to_string(), 7)?),
    };
    let tags_json: String = row.get(9)?;
    let tags = serde_json::from_str::<Vec<String>>(&tags_json).map_err(|_| {
        rusqlite::Error::InvalidColumnType(9, tags_json, rusqlite::types::Type::Text)
    })?;
    let description = empty_string_to_none(row.get::<_, String>(3)?);
    let assigned_to_turn = empty_string_to_none(row.get::<_, String>(8)?);

    Ok(TodoItem {
        id,
        session_id,
        title: row.get(2)?,
        description,
        status: TodoStatus::from_str(&row.get::<_, String>(4)?),
        priority: TodoPriority::from_str(&row.get::<_, String>(5)?),
        created_at,
        completed_at,
        assigned_to_turn,
        tags,
    })
}

fn parse_uuid_column(value: String, column: usize) -> RusqliteResult<Uuid> {
    Uuid::parse_str(&value)
        .map_err(|_| rusqlite::Error::InvalidColumnType(column, value, rusqlite::types::Type::Text))
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
