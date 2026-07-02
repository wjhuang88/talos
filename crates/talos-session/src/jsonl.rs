use crate::{Session, SessionEntry, SessionError, SessionMetadata};
use chrono::Utc;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;
use talos_core::message::{AgentEvent, Message};
use uuid::Uuid;

impl Session {
    pub fn append(&self, message: &Message) -> Result<(), SessionError> {
        self.append_with_metadata(message, SessionMetadata::default())
    }

    pub fn append_with_metadata(
        &self,
        message: &Message,
        metadata: SessionMetadata,
    ) -> Result<(), SessionError> {
        let (role, content) = message_parts(message);
        let entry = self.build_entry(&role, &content, metadata)?;
        self.append_entry_locked(&entry)
    }

    pub fn append_event(&self, event: &AgentEvent) -> Result<(), SessionError> {
        if matches!(event, AgentEvent::ThinkingDelta { .. }) {
            return Ok(());
        }
        let content =
            serde_json::to_string(event).map_err(|e| SessionError::InvalidJson(e.to_string()))?;
        let entry = self.build_entry("system", &content, SessionMetadata::default())?;
        self.append_entry_locked(&entry)
    }

    fn build_entry(
        &self,
        role: &str,
        content: &str,
        metadata: SessionMetadata,
    ) -> Result<SessionEntry, SessionError> {
        let parent_id = {
            let guard = self
                .last_entry_id
                .lock()
                .expect("last_entry_id mutex poisoned");
            if guard.is_none() {
                drop(guard);
                let id = read_last_entry_id(&self.file_path);
                *self
                    .last_entry_id
                    .lock()
                    .expect("last_entry_id mutex poisoned") = id.clone();
                id
            } else {
                guard.clone()
            }
        };

        Ok(SessionEntry {
            id: Uuid::new_v4().to_string(),
            parent_id,
            timestamp: Utc::now(),
            role: role.to_string(),
            content: content.to_string(),
            metadata,
        })
    }

    fn append_entry_locked(&self, entry: &SessionEntry) -> Result<(), SessionError> {
        let _lock = self.write_lock.lock().expect("write_lock mutex poisoned");
        let line =
            serde_json::to_string(entry).map_err(|e| SessionError::InvalidJson(e.to_string()))?;

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

        *self
            .last_entry_id
            .lock()
            .expect("last_entry_id mutex poisoned") = Some(entry.id.clone());
        Ok(())
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
                "assistant" => {
                    let tool_calls =
                        talos_core::message::extract_tool_calls_from_text(&entry.content);
                    let cleaned = talos_core::message::strip_tool_syntax(&entry.content);
                    Message::Assistant {
                        content: cleaned,
                        tool_calls,
                    }
                    .into()
                }
                "system" => {
                    if let Some(sys_content) = entry.content.strip_prefix("__SYSTEM__:") {
                        Some(Message::System {
                            content: sys_content.to_string(),
                            cache_markers: Vec::new(),
                        })
                    } else if serde_json::from_str::<AgentEvent>(&entry.content).is_ok() {
                        None
                    } else {
                        let (is_error, tool_use_id, content) = parse_tool_result(&entry.content);
                        Some(Message::Tool {
                            result: talos_core::message::MessageToolResult {
                                tool_use_id,
                                content,
                                is_error,
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
}

fn read_last_entry_id(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let file_size = file.metadata().ok()?.len();
    if file_size == 0 {
        return None;
    }
    let read_size = std::cmp::min(file_size, 8192) as usize;
    let seek_pos = file_size.saturating_sub(read_size as u64);
    file.seek(SeekFrom::Start(seek_pos)).ok()?;
    let mut buf = vec![0u8; read_size];
    file.read_exact(&mut buf).ok()?;
    let text = String::from_utf8_lossy(&buf);
    let last_line = text.lines().rev().find(|l| !l.is_empty())?;
    let entry: SessionEntry = serde_json::from_str(last_line).ok()?;
    Some(entry.id)
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

fn parse_tool_result(content: &str) -> (bool, String, String) {
    if let Some(rest) = content.strip_prefix("__ERROR__:")
        && let Some((id, body)) = rest.split_once("__\n")
    {
        return (true, id.to_string(), body.to_string());
    }
    if let Some(rest) = content.strip_prefix("__OK__:")
        && let Some((id, body)) = rest.split_once("__\n")
    {
        return (false, id.to_string(), body.to_string());
    }
    (false, "unknown".to_string(), content.to_string())
}

fn message_parts(message: &Message) -> (String, String) {
    match message {
        Message::User { content } => ("user".to_string(), content.clone()),
        Message::Assistant {
            content,
            tool_calls,
        } => {
            if tool_calls.is_empty() {
                return ("assistant".to_string(), content.clone());
            }
            // Embed tool calls as json-tool blocks so they survive JSONL round-trip.
            let mut full = content.clone();
            for tc in tool_calls {
                let block = serde_json::json!({
                    "name": tc.name,
                    "args": tc.input,
                });
                full.push_str(&format!("\n```json-tool\n{block}\n```"));
            }
            ("assistant".to_string(), full)
        }
        Message::Tool { result } => {
            let prefix = if result.is_error {
                format!("__ERROR__:{}__\n", result.tool_use_id)
            } else {
                format!("__OK__:{}__\n", result.tool_use_id)
            };
            ("system".to_string(), format!("{prefix}{}", result.content))
        }
        Message::System { content, .. } => ("system".to_string(), format!("__SYSTEM__:{content}")),
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
