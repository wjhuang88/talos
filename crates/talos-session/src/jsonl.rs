use crate::{Session, SessionEntry, SessionError, SessionMetadata};
use chrono::Utc;
use std::collections::HashSet;
use talos_core::message::{AgentEvent, Message};
use uuid::Uuid;

impl Session {
    pub fn append(&self, message: &Message) -> Result<(), SessionError> {
        self.append_with_metadata(message, SessionMetadata::default())
    }

    pub fn append_with_metadata(
        &self,
        message: &Message,
        mut metadata: SessionMetadata,
    ) -> Result<(), SessionError> {
        let (role, content) = message_parts(message);
        if let Message::Assistant {
            reasoning: Some(r), ..
        } = message
        {
            metadata.reasoning = Some(r.clone());
        }
        let entry = self.build_entry(&role, &content, metadata)?;
        self.append_entry_locked(&entry)
    }

    pub fn append_event(&self, event: &AgentEvent) -> Result<(), SessionError> {
        if matches!(
            event,
            AgentEvent::ThinkingDelta { .. } | AgentEvent::ReasoningComplete { .. }
        ) {
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
                let id = self.store.read_last_entry_id(&self.file_path);
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
        self.store.append_entry(&self.file_path, entry)
    }

    pub fn read_entries(&self) -> Result<Vec<SessionEntry>, SessionError> {
        self.store.read_entries(&self.file_path)
    }

    pub fn read_messages(&self) -> Result<Vec<Message>, SessionError> {
        let entries = self.read_entries()?;
        let mut messages = Vec::new();
        let mut pending_tool_call_ids = HashSet::new();

        for entry in entries {
            let msg = match entry.role.as_str() {
                "user" => {
                    pending_tool_call_ids.clear();
                    Some(Message::User {
                        content: entry.content,
                    })
                }
                "assistant" => {
                    let tool_calls =
                        talos_core::message::extract_tool_calls_from_text(&entry.content);
                    let cleaned = talos_core::message::strip_tool_syntax(&entry.content);
                    pending_tool_call_ids.clear();
                    pending_tool_call_ids.extend(tool_calls.iter().map(|call| call.id.clone()));
                    Message::Assistant {
                        content: cleaned,
                        tool_calls,
                        reasoning: entry.metadata.reasoning,
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
                        if pending_tool_call_ids.remove(&tool_use_id) {
                            Some(Message::Tool {
                                result: talos_core::message::MessageToolResult {
                                    tool_use_id,
                                    content,
                                    is_error,
                                },
                            })
                        } else {
                            None
                        }
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

pub(crate) fn parse_tool_result(content: &str) -> (bool, String, String) {
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

pub(crate) fn message_parts(message: &Message) -> (String, String) {
    match message {
        Message::User { content } => ("user".to_string(), content.clone()),
        Message::Assistant {
            content,
            tool_calls,
            ..
        } => {
            if tool_calls.is_empty() {
                return ("assistant".to_string(), content.clone());
            }
            let mut full = content.clone();
            for tc in tool_calls {
                let block = serde_json::json!({
                    "id": tc.id,
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

pub(crate) fn preview_text(content: &str) -> String {
    const MAX_PREVIEW_CHARS: usize = 100;
    let mut chars = content.chars();
    let preview: String = chars.by_ref().take(MAX_PREVIEW_CHARS).collect();
    if chars.next().is_some() {
        format!("{preview}...")
    } else {
        preview
    }
}
