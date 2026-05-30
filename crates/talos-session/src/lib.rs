//! Talos session management — JSONL-based session logging.
//!
//! Sessions are stored as append-only JSONL files, organized by working directory.
//! Each line in a JSONL file is a JSON object with a `type` field (`"message"` or `"event"`)
//! and a `data` field containing the serialized payload.
//!
//! # Directory Layout
//!
//! ```text
//! ~/.talos/sessions/
//!   <project>/
//!     <uuid>.jsonl
//! ```
//!
//! # Crash Safety
//!
//! JSONL is append-only. If a crash occurs, only the last line may be corrupted,
//! which can be detected and skipped during reads.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

use talos_core::message::{AgentEvent, Message};

/// Errors that can occur during session operations.
#[derive(Debug, Error)]
pub enum SessionError {
    /// An I/O error occurred (file read/write, directory creation, etc.).
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// A line in the JSONL file is not valid JSON.
    #[error("invalid JSON in session file: {0}")]
    InvalidJson(String),

    /// The requested session was not found.
    #[error("session not found: {0}")]
    SessionNotFound(Uuid),
}

/// Metadata about a session, returned when listing sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    /// Unique session identifier.
    pub id: Uuid,
    /// Project name or working directory path.
    pub project: String,
    /// When the session was created.
    pub created_at: DateTime<Utc>,
    /// Number of messages in the session.
    pub message_count: usize,
    /// Path to the JSONL file.
    pub file_path: PathBuf,
}

/// A single session, backed by a JSONL file.
#[derive(Debug, Clone)]
pub struct Session {
    /// Unique session identifier.
    pub id: Uuid,
    /// Project name or working directory path.
    pub project: String,
    /// When the session was created.
    pub created_at: DateTime<Utc>,
    /// Path to the JSONL file backing this session.
    pub file_path: PathBuf,
}

impl Session {
    /// Append a message to the session's JSONL file.
    pub fn append(&self, message: &Message) -> Result<(), SessionError> {
        let entry = serde_json::json!({
            "type": "message",
            "data": message,
        });
        let line = serde_json::to_string(&entry).map_err(|e| SessionError::InvalidJson(e.to_string()))?;
        self.append_line(&line)
    }

    /// Append an agent event to the session's JSONL file.
    pub fn append_event(&self, event: &AgentEvent) -> Result<(), SessionError> {
        let entry = serde_json::json!({
            "type": "event",
            "data": event,
        });
        let line = serde_json::to_string(&entry).map_err(|e| SessionError::InvalidJson(e.to_string()))?;
        self.append_line(&line)
    }

    /// Read all messages from the session's JSONL file.
    ///
    /// Only lines with `"type": "message"` are returned; event lines are skipped.
    /// Corrupted lines are silently skipped (crash-safety guarantee).
    pub fn read_messages(&self) -> Result<Vec<Message>, SessionError> {
        let file = fs::File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut messages = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            // Try to parse as a session entry with Message data
            if let Ok(entry) = serde_json::from_str::<serde_json::Value>(&line) {
                if entry.get("type").and_then(|t| t.as_str()) == Some("message") {
                    if let Some(data) = entry.get("data") {
                        if let Ok(msg) = serde_json::from_value::<Message>(data.clone()) {
                            messages.push(msg);
                        }
                        // If data doesn't parse as Message, skip (corrupted line)
                    }
                }
            }
            // If the line isn't valid JSON at all, skip it
        }

        Ok(messages)
    }

    /// Read all events from the session's JSONL file.
    ///
    /// Only lines with `"type": "event"` are returned; message lines are skipped.
    pub fn read_events(&self) -> Result<Vec<AgentEvent>, SessionError> {
        let file = fs::File::open(&self.file_path)?;
        let reader = BufReader::new(file);
        let mut events = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<serde_json::Value>(&line) {
                if entry.get("type").and_then(|t| t.as_str()) == Some("event") {
                    if let Some(data) = entry.get("data") {
                        if let Ok(event) = serde_json::from_value::<AgentEvent>(data.clone()) {
                            events.push(event);
                        }
                    }
                }
            }
        }

        Ok(events)
    }

    /// Append a raw JSON line to the session file.
    fn append_line(&self, line: &str) -> Result<(), SessionError> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;
        writeln!(file, "{line}")?;
        Ok(())
    }
}

/// Manages sessions on disk.
#[derive(Debug, Clone)]
pub struct SessionManager {
    /// Root directory for all session files.
    sessions_dir: PathBuf,
}

impl SessionManager {
    /// Create a new `SessionManager` with the default sessions directory (`~/.talos/sessions/`).
    pub fn new() -> Result<Self, SessionError> {
        let home = std::env::var("HOME").map_err(|e| SessionError::IoError(std::io::Error::other(e)))?;
        let dir = PathBuf::from(home).join(".talos").join("sessions");
        Ok(Self { sessions_dir: dir })
    }

    /// Create a new `SessionManager` with a custom sessions directory.
    pub fn with_dir(sessions_dir: PathBuf) -> Self {
        Self { sessions_dir }
    }

    /// Create a new session for the given project.
    ///
    /// The session file is created at `~/.talos/sessions/<project>/<uuid>.jsonl`.
    pub fn create_session(&self, project: &str) -> Result<Session, SessionError> {
        let id = Uuid::new_v4();
        let project_dir = self.sessions_dir.join(project);
        fs::create_dir_all(&project_dir)?;

        let file_path = project_dir.join(format!("{id}.jsonl"));
        // Create the file (empty)
        fs::File::create(&file_path)?;

        Ok(Session {
            id,
            project: project.to_string(),
            created_at: Utc::now(),
            file_path,
        })
    }

    /// Load an existing session by ID.
    ///
    /// Scans all project directories for a file matching `<id>.jsonl`.
    pub fn get_session(&self, id: &Uuid) -> Result<Session, SessionError> {
        if !self.sessions_dir.exists() {
            return Err(SessionError::SessionNotFound(*id));
        }

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let project_dir = entry.path();
            let project_name = project_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let file_path = project_dir.join(format!("{id}.jsonl"));
            if file_path.exists() {
                let metadata = fs::metadata(&file_path)?;
                let created_at = metadata
                    .modified()
                    .ok()
                    .map(DateTime::<Utc>::from)
                    .unwrap_or_else(Utc::now);

                return Ok(Session {
                    id: *id,
                    project: project_name,
                    created_at,
                    file_path,
                });
            }
        }

        Err(SessionError::SessionNotFound(*id))
    }

    /// List all sessions across all project directories.
    pub fn list_sessions(&self) -> Result<Vec<SessionMeta>, SessionError> {
        let mut sessions = Vec::new();

        if !self.sessions_dir.exists() {
            return Ok(sessions);
        }

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let project_dir = entry.path();
            let project_name = project_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            for file_entry in fs::read_dir(&project_dir)? {
                let file_entry = file_entry?;
                let path = file_entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                    continue;
                }

                let file_stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .and_then(|s| Uuid::parse_str(s).ok());

                if let Some(id) = file_stem {
                    let metadata = fs::metadata(&path)?;
                    let created_at = metadata
                        .modified()
                        .ok()
                        .map(DateTime::<Utc>::from)
                        .unwrap_or_else(Utc::now);

                    let message_count = Self::count_messages_in_file(&path)?;

                    sessions.push(SessionMeta {
                        id,
                        project: project_name.clone(),
                        created_at,
                        message_count,
                        file_path: path,
                    });
                }
            }
        }

        Ok(sessions)
    }

    /// Count the number of message entries in a JSONL file.
    fn count_messages_in_file(path: &Path) -> Result<usize, SessionError> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<serde_json::Value>(&line) {
                if entry.get("type").and_then(|t| t.as_str()) == Some("message") {
                    count += 1;
                }
            }
        }

        Ok(count)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        // Use a fallback path if HOME is not set
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        Self {
            sessions_dir: PathBuf::from(home).join(".talos").join("sessions"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use talos_core::message::{StopReason, ToolCall, ToolResult, Usage};

    fn test_manager() -> SessionManager {
        let dir = tempfile::tempdir().unwrap();
        SessionManager::with_dir(dir.path().to_path_buf())
    }

    #[test]
    fn create_session_creates_file() {
        let manager = test_manager();
        let session = manager.create_session("test-project").unwrap();

        assert!(session.file_path.exists());
        assert_eq!(session.project, "test-project");
        assert!(session.file_path.to_string_lossy().ends_with(".jsonl"));
    }

    #[test]
    fn create_session_uses_correct_directory() {
        let manager = test_manager();
        let session = manager.create_session("my-project").unwrap();

        let expected_dir = manager.sessions_dir.join("my-project");
        assert!(session.file_path.starts_with(expected_dir));
    }

    #[test]
    fn append_and_read_messages() {
        let manager = test_manager();
        let session = manager.create_session("test-project").unwrap();

        let msg1 = Message::User {
            content: "Hello!".into(),
        };
        let msg2 = Message::Assistant {
            content: "Hi there!".into(),
            tool_calls: vec![],
        };

        session.append(&msg1).unwrap();
        session.append(&msg2).unwrap();

        let messages = session.read_messages().unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], msg1);
        assert_eq!(messages[1], msg2);
    }

    #[test]
    fn append_and_read_events() {
        let manager = test_manager();
        let session = manager.create_session("test-project").unwrap();

        let event1 = AgentEvent::TurnStart;
        let event2 = AgentEvent::TextDelta {
            delta: "Hello".into(),
        };
        let event3 = AgentEvent::TurnEnd {
            stop_reason: StopReason::EndTurn,
            usage: Usage::default(),
        };

        session.append_event(&event1).unwrap();
        session.append_event(&event2).unwrap();
        session.append_event(&event3).unwrap();

        let events = session.read_events().unwrap();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0], event1);
        assert_eq!(events[1], event2);
        assert_eq!(events[2], event3);
    }

    #[test]
    fn read_messages_skips_events() {
        let manager = test_manager();
        let session = manager.create_session("test-project").unwrap();

        let msg = Message::User {
            content: "test".into(),
        };
        let event = AgentEvent::TurnStart;

        session.append(&msg).unwrap();
        session.append_event(&event).unwrap();

        let messages = session.read_messages().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], msg);
    }

    #[test]
    fn list_sessions() {
        let manager = test_manager();

        let s1 = manager.create_session("project-a").unwrap();
        let s2 = manager.create_session("project-b").unwrap();

        // Append a message to s1 so it has a count
        s1.append(&Message::User {
            content: "msg".into(),
        })
        .unwrap();

        let sessions = manager.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);

        let ids: Vec<Uuid> = sessions.iter().map(|s| s.id).collect();
        assert!(ids.contains(&s1.id));
        assert!(ids.contains(&s2.id));

        let s1_meta = sessions.iter().find(|s| s.id == s1.id).unwrap();
        assert_eq!(s1_meta.message_count, 1);

        let s2_meta = sessions.iter().find(|s| s.id == s2.id).unwrap();
        assert_eq!(s2_meta.message_count, 0);
    }

    #[test]
    fn get_session_existing() {
        let manager = test_manager();
        let session = manager.create_session("test-project").unwrap();
        let id = session.id;

        let loaded = manager.get_session(&id).unwrap();
        assert_eq!(loaded.id, id);
        assert_eq!(loaded.project, "test-project");
    }

    #[test]
    fn get_session_not_found() {
        let manager = test_manager();
        let fake_id = Uuid::new_v4();

        let result = manager.get_session(&fake_id);
        assert!(result.is_err());
        match result.unwrap_err() {
            SessionError::SessionNotFound(id) => assert_eq!(id, fake_id),
            other => panic!("expected SessionNotFound, got {other:?}"),
        }
    }

    #[test]
    fn invalid_json_lines_are_skipped() {
        let manager = test_manager();
        let session = manager.create_session("test-project").unwrap();

        // Write a valid message
        session
            .append(&Message::User {
                content: "valid".into(),
            })
            .unwrap();

        // Manually append an invalid JSON line
        let mut file = OpenOptions::new()
            .append(true)
            .open(&session.file_path)
            .unwrap();
        writeln!(file, "this is not json").unwrap();

        // Append another valid message
        session
            .append(&Message::User {
                content: "also valid".into(),
            })
            .unwrap();

        let messages = session.read_messages().unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].clone(), Message::User {
            content: "valid".into(),
        });
        assert_eq!(messages[1].clone(), Message::User {
            content: "also valid".into(),
        });
    }

    #[test]
    fn list_sessions_empty_directory() {
        let manager = test_manager();
        let sessions = manager.list_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn session_with_tool_calls() {
        let manager = test_manager();
        let session = manager.create_session("test-project").unwrap();

        let msg = Message::Assistant {
            content: "Let me check that file.".into(),
            tool_calls: vec![ToolCall {
                id: "call_1".into(),
                name: "read_file".into(),
                input: serde_json::json!({"path": "src/main.rs"}),
            }],
        };

        session.append(&msg).unwrap();

        let messages = session.read_messages().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], msg);
    }

    #[test]
    fn session_with_tool_result() {
        let manager = test_manager();
        let session = manager.create_session("test-project").unwrap();

        let msg = Message::Tool {
            result: ToolResult {
                tool_use_id: "call_1".into(),
                content: "fn main() {}".into(),
                is_error: false,
            },
        };

        session.append(&msg).unwrap();

        let messages = session.read_messages().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], msg);
    }
}
