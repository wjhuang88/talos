use crate::{Session, SessionEntry, SessionError, SessionMetadata};
use chrono::Utc;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use talos_core::message::{AgentEvent, Message};
use uuid::Uuid;

impl Session {
    /// Append a message to the current branch and persist it to the JSONL file.
    pub fn append(&self, message: &Message) -> Result<(), SessionError> {
        let (role, content) = message_parts(message);
        let entries = self.read_entries()?;
        let parent_id = entries.last().map(|e| e.id.clone());

        let entry = SessionEntry {
            id: Uuid::new_v4().to_string(),
            parent_id,
            timestamp: Utc::now(),
            role,
            content,
            metadata: SessionMetadata::default(),
        };

        self.append_entry(&entry)
    }

    /// Append an agent event to the current branch and persist it to the JSONL file.
    pub fn append_event(&self, event: &AgentEvent) -> Result<(), SessionError> {
        let content =
            serde_json::to_string(event).map_err(|e| SessionError::InvalidJson(e.to_string()))?;
        let role = "system".to_string();

        let entries = self.read_entries()?;
        let parent_id = entries.last().map(|e| e.id.clone());

        let entry = SessionEntry {
            id: Uuid::new_v4().to_string(),
            parent_id,
            timestamp: Utc::now(),
            role,
            content,
            metadata: SessionMetadata::default(),
        };

        self.append_entry(&entry)
    }

    /// Read all entries from the session's JSONL file.
    ///
    /// Entries are reconstructed from the JSONL format. Entries without `id` or
    /// `parent_id` (backward compatibility) are assigned synthetic IDs and treated
    /// as a single linear branch.
    pub fn read_entries(&self) -> Result<Vec<SessionEntry>, SessionError> {
        read_entries_from_path(&self.file_path)
    }

    /// Read all messages from the session's JSONL file for the current branch.
    ///
    /// Only entries with role `"user"`, `"assistant"`, or `"system"` that contain
    /// valid message data are returned.
    pub fn read_messages(&self) -> Result<Vec<Message>, SessionError> {
        let entries = self.read_entries()?;
        let mut messages = Vec::new();

        for entry in entries {
            let msg = match entry.role.as_str() {
                "user" => Some(Message::User {
                    content: entry.content,
                }),
                "assistant" => Message::Assistant {
                    content: entry.content,
                    tool_calls: vec![],
                }
                .into(),
                "system" => {
                    // Try to parse as AgentEvent first
                    if serde_json::from_str::<AgentEvent>(&entry.content).is_ok() {
                        // System events are not messages, skip
                        None
                    } else {
                        // Treat as tool result
                        Some(Message::Tool {
                            result: talos_core::message::MessageToolResult {
                                tool_use_id: "unknown".to_string(),
                                content: entry.content,
                                is_error: false,
                            },
                        })
                    }
                }
                _ => None,
            };

            if let Some(msg) = msg {
                messages.push(msg);
            }
        }

        Ok(messages)
    }

    /// Read all events from the session's JSONL file.
    pub fn read_events(&self) -> Result<Vec<AgentEvent>, SessionError> {
        let entries = self.read_entries()?;
        let mut events = Vec::new();

        for entry in entries {
            if entry.role == "system"
                && let Ok(event) = serde_json::from_str::<AgentEvent>(&entry.content)
            {
                events.push(event);
            }
        }

        Ok(events)
    }

    /// Append a raw [`SessionEntry`] to the JSONL file.
    /// Creates the parent directory and file if this is the first append
    /// (supports deferred session creation).
    fn append_entry(&self, entry: &SessionEntry) -> Result<(), SessionError> {
        let line =
            serde_json::to_string(entry).map_err(|e| SessionError::InvalidJson(e.to_string()))?;

        // Ensure file and parent directory exist (lazy creation for deferred sessions).
        if !self.file_path.exists()
            && let Some(parent) = self.file_path.parent()
        {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)?;
        writeln!(file, "{line}")?;
        Ok(())
    }
}

pub(crate) fn scan_file(path: &Path) -> Result<(usize, String), SessionError> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut count = 0;
    let mut last_preview = String::new();

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<SessionEntry>(&line) {
            count += 1;
            last_preview = preview_text(&entry.content);
            continue;
        }

        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line)
            && value.get("type").and_then(|t| t.as_str()) == Some("message")
        {
            count += 1;
            if let Some(data) = value.get("data")
                && let Ok(msg) = serde_json::from_value::<Message>(data.clone())
            {
                let (_, content) = message_parts(&msg);
                last_preview = preview_text(&content);
            }
        }
    }

    Ok((count, last_preview))
}

fn read_entries_from_path(path: &Path) -> Result<Vec<SessionEntry>, SessionError> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();
    let mut synthetic_counter: u64 = 0;

    for line in reader.lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<SessionEntry>(&line) {
            entries.push(entry);
            continue;
        }

        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line)
            && value.get("type").and_then(|t| t.as_str()) == Some("message")
            && let Some(data) = value.get("data")
            && let Ok(msg) = serde_json::from_value::<Message>(data.clone())
        {
            let (role, content) = message_parts(&msg);
            let id = format!("synthetic-{synthetic_counter}");
            let parent_id = if synthetic_counter > 0 {
                Some(format!("synthetic-{}", synthetic_counter - 1))
            } else {
                None
            };

            entries.push(SessionEntry {
                id,
                parent_id,
                timestamp: Utc::now(),
                role,
                content,
                metadata: SessionMetadata::default(),
            });
            synthetic_counter += 1;
        }
        // Invalid lines are silently skipped (crash-safety guarantee)
    }

    Ok(entries)
}

fn message_parts(message: &Message) -> (String, String) {
    match message {
        Message::User { content } => ("user".to_string(), content.clone()),
        Message::Assistant { content, .. } => ("assistant".to_string(), content.clone()),
        Message::Tool { result } => ("system".to_string(), result.content.clone()),
        Message::System { content, .. } => ("system".to_string(), content.clone()),
        Message::Context { content } => ("user".to_string(), content.clone()),
    }
}

fn preview_text(content: &str) -> String {
    const MAX_PREVIEW_CHARS: usize = 100;
    let mut chars = content.chars();
    let preview: String = chars.by_ref().take(MAX_PREVIEW_CHARS).collect();
    if chars.next().is_some() {
        format!("{preview}...")
    } else {
        preview
    }
}
