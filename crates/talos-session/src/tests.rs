use super::*;
use crate::topology::workspace_dir_name;
use chrono::Utc;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use talos_core::message::{AgentEvent, Message, MessageToolResult, StopReason, ToolCall, Usage};
use uuid::Uuid;

fn test_manager() -> SessionManager {
    let dir = tempfile::tempdir().unwrap();
    SessionManager::with_dir(dir.path().to_path_buf())
}

#[test]
fn create_session_creates_file() {
    let manager = test_manager();
    let session = manager.create_session("test-project", "").unwrap();

    assert!(session.file_path.exists());
    assert_eq!(session.project, "test-project");
    assert!(session.file_path.to_string_lossy().ends_with(".jsonl"));
}

#[test]
fn create_session_uses_correct_directory() {
    let manager = test_manager();
    let session = manager.create_session("my-project", "my-project").unwrap();

    let expected_dir = manager.sessions_dir.join(workspace_dir_name("my-project"));
    assert!(session.file_path.starts_with(expected_dir));
}

#[test]
fn append_and_read_messages() {
    let manager = test_manager();
    let session = manager.create_session("test-project", "").unwrap();

    let msg1 = Message::User {
        content: "Hello!".into(),
    };
    let msg2 = Message::Assistant {
        content: "Hi there!".into(),
        tool_calls: vec![],
        reasoning: None,
    };

    session.append(&msg1).unwrap();
    session.append(&msg2).unwrap();

    let messages = session.read_messages().unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0], msg1);
    assert_eq!(
        messages[1],
        Message::Assistant {
            content: "Hi there!".into(),
            tool_calls: vec![],
            reasoning: None,
        }
    );
}

#[test]
fn append_event_ignores_transient_thinking_delta() {
    let manager = test_manager();
    let session = manager.create_session("test-project", "").unwrap();

    session
        .append_event(&AgentEvent::ThinkingDelta {
            delta: "private reasoning".to_string(),
        })
        .unwrap();

    let entries = session.read_entries().unwrap();
    assert!(entries.is_empty());
}

#[test]
fn resume_history_excludes_transient_thinking_delta() {
    let manager = test_manager();
    let session = manager.create_session("test-project", "").unwrap();

    session
        .append_event(&AgentEvent::ThinkingDelta {
            delta: "private reasoning".to_string(),
        })
        .unwrap();
    session
        .append(&Message::Assistant {
            content: "Final answer".to_string(),
            tool_calls: vec![],
            reasoning: None,
        })
        .unwrap();

    let resumed = manager.get_session(&session.id).unwrap();
    let messages = resumed.read_messages().unwrap();
    assert_eq!(
        messages,
        vec![Message::Assistant {
            content: "Final answer".to_string(),
            tool_calls: vec![],
            reasoning: None,
        }]
    );
}

#[test]
fn append_and_read_events() {
    let manager = test_manager();
    let session = manager.create_session("test-project", "").unwrap();

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
    let session = manager.create_session("test-project", "").unwrap();

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

    let s1 = manager.create_session("project-a", "").unwrap();
    let s2 = manager.create_session("project-b", "").unwrap();

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

    let s1_info = sessions.iter().find(|s| s.id == s1.id).unwrap();
    assert_eq!(s1_info.message_count, 1);
    assert!(!s1_info.last_message_preview.is_empty());

    let s2_info = sessions.iter().find(|s| s.id == s2.id).unwrap();
    assert_eq!(s2_info.message_count, 0);
}

#[test]
fn list_workspace_sessions_filters_by_workspace() {
    let manager = test_manager();
    let playit = manager.create_session("playit", "playit").unwrap();
    let talos = manager.create_session("talos", "").unwrap();

    playit
        .append(&Message::User {
            content: "playit message".into(),
        })
        .unwrap();
    talos
        .append(&Message::User {
            content: "talos message".into(),
        })
        .unwrap();

    let sessions = manager.list_workspace_sessions("playit").unwrap();

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, playit.id);
    assert_eq!(sessions[0].last_message_preview, "playit message");
}

#[test]
fn latest_workspace_session_returns_most_recent_session() {
    let manager = test_manager();
    let older = manager.create_session("playit", "playit").unwrap();
    let newer = manager.create_session("playit", "playit").unwrap();

    older
        .append(&Message::User {
            content: "older".into(),
        })
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));
    newer
        .append(&Message::User {
            content: "newer".into(),
        })
        .unwrap();

    let latest = manager
        .latest_workspace_session("playit")
        .unwrap()
        .expect("expected latest session");

    assert_eq!(latest.id, newer.id);
    assert_eq!(latest.last_message_preview, "newer");
}

#[test]
fn latest_workspace_session_returns_none_for_empty_workspace() {
    let manager = test_manager();

    let latest = manager.latest_workspace_session("missing").unwrap();

    assert!(latest.is_none());
}

#[test]
fn get_session_existing() {
    let manager = test_manager();
    let session = manager.create_session("test-project", "").unwrap();
    let id = session.id;

    let loaded = manager.get_session(&id).unwrap();
    assert_eq!(loaded.id, id);
    // project name may differ from display_name on disk readback (MEM-004 hash dirs)
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
    let session = manager.create_session("test-project", "").unwrap();

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
    assert_eq!(
        messages[0].clone(),
        Message::User {
            content: "valid".into(),
        }
    );
    assert_eq!(
        messages[1].clone(),
        Message::User {
            content: "also valid".into(),
        }
    );
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
    let session = manager.create_session("test-project", "").unwrap();

    let msg = Message::Assistant {
        content: "Let me check that file.".into(),
        tool_calls: vec![ToolCall {
            id: "call_1".into(),
            name: "read_file".into(),
            input: serde_json::json!({"path": "src/main.rs"}),
        }],
        reasoning: None,
    };

    session.append(&msg).unwrap();

    let messages = session.read_messages().unwrap();
    assert_eq!(messages.len(), 1);
    match &messages[0] {
        Message::Assistant {
            content,
            tool_calls,
            ..
        } => {
            assert_eq!(content, "Let me check that file.");
            assert_eq!(tool_calls.len(), 1);
            assert_eq!(tool_calls[0].id, "call_1");
            assert_eq!(tool_calls[0].name, "read_file");
            assert_eq!(
                tool_calls[0].input,
                serde_json::json!({"path": "src/main.rs"})
            );
        }
        _ => panic!("expected Assistant message"),
    }
}

#[test]
fn session_tool_call_id_matches_tool_result_id_after_resume() {
    let manager = test_manager();
    let session = manager.create_session("test-project", "").unwrap();

    session
        .append(&Message::User {
            content: "list files".into(),
        })
        .unwrap();
    session
        .append(&Message::Assistant {
            content: String::new(),
            tool_calls: vec![ToolCall {
                id: "call_abc123".into(),
                name: "bash".into(),
                input: serde_json::json!({"command": "ls"}),
            }],
            reasoning: None,
        })
        .unwrap();
    session
        .append(&Message::Tool {
            result: MessageToolResult {
                tool_use_id: "call_abc123".into(),
                content: "file1.rs\nfile2.rs".into(),
                is_error: false,
            },
        })
        .unwrap();

    let messages = session.read_messages().unwrap();
    assert_eq!(messages.len(), 3);

    let assistant_tool_id = match &messages[1] {
        Message::Assistant { tool_calls, .. } => &tool_calls[0].id,
        _ => panic!("expected Assistant message at index 1"),
    };
    let tool_use_id = match &messages[2] {
        Message::Tool { result } => &result.tool_use_id,
        _ => panic!("expected Tool message at index 2"),
    };
    assert_eq!(
        assistant_tool_id, tool_use_id,
        "assistant tool_call.id must match tool result tool_use_id for provider round-trip"
    );
}

#[test]
fn ensure_persisted_creates_empty_file_for_deferred_session() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("session.jsonl");
    let mut session = Session::new_deferred(
        Uuid::new_v4(),
        "test".into(),
        String::new(),
        file_path.clone(),
    );
    assert!(!session.persisted);
    assert!(!file_path.exists());

    session.ensure_persisted().unwrap();

    assert!(session.persisted);
    assert!(file_path.exists());
}

#[test]
fn ensure_persisted_does_not_truncate_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("session.jsonl");

    // Simulate: another Session clone (e.g., from a watch channel) already
    // created and wrote to the file via `append_with_metadata`. The clone
    // taken here still has `persisted = false` (the flag is a plain bool
    // copied on clone, not shared). `ensure_persisted` must NOT truncate
    // the existing content. This is the regression for the model-switch
    // context-loss bug — see EVOLUTION.md lesson #30 lineage.
    let mut prior = Session::new_deferred(
        Uuid::new_v4(),
        "test".into(),
        String::new(),
        file_path.clone(),
    );
    prior
        .append(&Message::User {
            content: "important prior turn".into(),
        })
        .unwrap();
    prior
        .append(&Message::Assistant {
            content: "important prior answer".into(),
            tool_calls: vec![],
            reasoning: None,
        })
        .unwrap();
    let prior_size = std::fs::metadata(&file_path).unwrap().len();
    assert!(prior_size > 0);

    let mut clone = prior.clone();
    assert!(
        !clone.persisted,
        "Session::Clone copies the persisted bool verbatim"
    );

    clone.ensure_persisted().unwrap();

    assert_eq!(
        std::fs::metadata(&file_path).unwrap().len(),
        prior_size,
        "ensure_persisted must not truncate the existing JSONL file"
    );
    let messages = clone.read_messages().unwrap();
    assert_eq!(messages.len(), 2, "prior history must survive the clone");
}

#[test]
fn model_switch_simulation_preserves_full_history() {
    // End-to-end regression for the user-reported bug: after multiple
    // turn cycles through a Session (simulating real TUI usage), a clone
    // through the watch channel must see the full history when used to
    // build a new AppServerSession — as `rebuild_session_for_model` does.
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("session.jsonl");

    // Stage 1: deferred session is created (TUI startup path).
    let mut session = Session::new_deferred(
        Uuid::new_v4(),
        "talos".into(),
        "/work".into(),
        file_path.clone(),
    );
    assert!(!session.persisted);

    // Stage 2: user sends the first message. Persistence happens via
    // a clone through the watch channel (user_msg_persister task).
    let clone_for_first_msg = session.clone();
    clone_for_first_msg
        .append(&Message::User {
            content: "summarize the repo".into(),
        })
        .unwrap();
    // The original session struct still has persisted = false (flag is
    // copied on clone, never updated by append_with_metadata).
    assert!(!session.persisted);
    // But the file does exist and has content.
    assert!(file_path.exists());
    assert!(std::fs::metadata(&file_path).unwrap().len() > 0);

    // Stage 3: agent runs, assistant message and tool result are
    // persisted through the bridge forwarder (also via watch-channel clones).
    let clone_for_assistant = session.clone();
    clone_for_assistant
        .append(&Message::Assistant {
            content: "I'll inspect the repo.".into(),
            tool_calls: vec![],
            reasoning: None,
        })
        .unwrap();

    // Stage 4: user invokes /model. `rebuild_session_for_model` clones
    // from the watch channel, calls ensure_persisted, then read_messages.
    // Before the fix, this truncated the file. After the fix, history
    // is preserved.
    let mut switch_clone = session.clone();
    switch_clone.ensure_persisted().unwrap();
    let history = switch_clone.read_messages().unwrap();

    assert_eq!(
        history.len(),
        2,
        "user + assistant messages must survive a model switch clone"
    );
    match &history[0] {
        Message::User { content } => assert_eq!(content, "summarize the repo"),
        other => panic!("expected User at index 0, got {other:?}"),
    }
    match &history[1] {
        Message::Assistant { content, .. } => {
            assert_eq!(content, "I'll inspect the repo.")
        }
        other => panic!("expected Assistant at index 1, got {other:?}"),
    }
}

#[test]
fn session_with_tool_result() {
    let manager = test_manager();
    let session = manager.create_session("test-project", "").unwrap();

    let msg = Message::Tool {
        result: MessageToolResult {
            tool_use_id: "call_1".into(),
            content: "fn main() {}".into(),
            is_error: false,
        },
    };

    session.append(&msg).unwrap();

    let messages = session.read_messages().unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(
        messages[0],
        Message::Tool {
            result: MessageToolResult {
                tool_use_id: "call_1".into(),
                content: "fn main() {}".into(),
                is_error: false,
            },
        }
    );
}

// === Branching Tests ===

#[test]
fn session_entry_with_parent_child_relationship() {
    let manager = test_manager();
    let session = manager.create_session("test-project", "").unwrap();

    let msg1 = Message::User {
        content: "Hello".into(),
    };
    let msg2 = Message::Assistant {
        content: "Hi".into(),
        tool_calls: vec![],
        reasoning: None,
    };

    session.append(&msg1).unwrap();
    session.append(&msg2).unwrap();

    let entries = session.read_entries().unwrap();
    assert_eq!(entries.len(), 2);

    // First entry has no parent
    assert!(entries[0].parent_id.is_none());
    // Second entry has parent_id pointing to first
    assert_eq!(entries[1].parent_id, Some(entries[0].id.clone()));
}

#[test]
fn fork_creates_new_branch_with_correct_parent_id() {
    let manager = test_manager();
    let mut session = manager.create_session("test-project", "").unwrap();

    // Add some messages
    session
        .append(&Message::User {
            content: "msg1".into(),
        })
        .unwrap();
    session
        .append(&Message::Assistant {
            content: "reply1".into(),
            tool_calls: vec![],
            reasoning: None,
        })
        .unwrap();
    session
        .append(&Message::User {
            content: "msg2".into(),
        })
        .unwrap();

    let entries = session.read_entries().unwrap();
    let fork_from_id = entries[1].id.clone(); // Fork from the assistant's reply

    let original_branch = session.current_branch.clone();
    let new_branch_id = session.fork(&fork_from_id).unwrap();

    // New branch should be different from original
    assert_ne!(new_branch_id, original_branch);
    assert_eq!(session.current_branch, new_branch_id);

    // New branch should have entries up to and including the fork point
    let new_branch = session.get_branch(&new_branch_id).unwrap();
    assert_eq!(new_branch.entries.len(), 2);
    assert_eq!(new_branch.root_id, fork_from_id);

    let all_entries = session.read_entries().unwrap();
    assert_eq!(all_entries.len(), 3);
}

#[test]
fn list_branches_returns_all_branch_ids() {
    let manager = test_manager();
    let mut session = manager.create_session("test-project", "").unwrap();

    // Add a message and fork
    session
        .append(&Message::User {
            content: "msg".into(),
        })
        .unwrap();

    let entries = session.read_entries().unwrap();
    session.fork(&entries[0].id).unwrap();

    let branches = session.list_branches();
    assert_eq!(branches.len(), 2);
}

#[test]
fn resume_session_loads_existing_jsonl_file() {
    let manager = test_manager();

    // Create and populate a session
    let session = manager.create_session("test-project", "").unwrap();
    let session_id = session.id.to_string();

    session
        .append(&Message::User {
            content: "Hello".into(),
        })
        .unwrap();
    session
        .append(&Message::Assistant {
            content: "Hi there".into(),
            tool_calls: vec![],
            reasoning: None,
        })
        .unwrap();

    // Resume the session
    let resumed = manager.resume_session(&session_id).unwrap();
    assert_eq!(resumed.id.to_string(), session_id);

    // Entries should be loaded
    let entries = resumed.read_entries().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].content, "Hello");
    assert_eq!(entries[1].content, "Hi there");
}

#[test]
fn list_sessions_preview_handles_utf8_char_boundary() {
    let manager = test_manager();
    let session = manager.create_session("test-project", "").unwrap();
    let content = "你好！我是 Talos，一个 AI 编程助手。".repeat(8);

    session.append(&Message::User { content }).unwrap();

    let sessions = manager.list_sessions().unwrap();
    let info = sessions
        .iter()
        .find(|info| info.id == session.id)
        .expect("session should be listed");
    assert!(info.last_message_preview.ends_with("..."));
    assert!(
        info.last_message_preview
            .is_char_boundary(info.last_message_preview.len())
    );
}

#[test]
fn list_sessions_old_format_preview_handles_utf8_char_boundary() {
    let manager = test_manager();
    let session = manager.create_session("test-project", "").unwrap();
    let content = "你好！我是 Talos，一个 AI 编程助手。".repeat(8);

    let mut file = OpenOptions::new()
        .append(true)
        .open(&session.file_path)
        .unwrap();
    let old_entry = serde_json::json!({
        "type": "message",
        "data": {
            "role": "user",
            "content": content
        }
    });
    writeln!(file, "{old_entry}").unwrap();

    let sessions = manager.list_sessions().unwrap();
    let info = sessions
        .iter()
        .find(|info| info.id == session.id)
        .expect("session should be listed");
    assert!(info.last_message_preview.ends_with("..."));
    assert!(
        info.last_message_preview
            .is_char_boundary(info.last_message_preview.len())
    );
}

#[test]
fn backward_compatibility_with_old_jsonl_format() {
    let manager = test_manager();
    let session = manager.create_session("test-project", "").unwrap();

    // Manually write old-format JSONL lines
    let mut file = OpenOptions::new()
        .append(true)
        .open(&session.file_path)
        .unwrap();

    let old_entry1 = serde_json::json!({
        "type": "message",
        "data": {
            "role": "user",
            "content": "Old format message 1"
        }
    });
    let old_entry2 = serde_json::json!({
        "type": "message",
        "data": {
            "role": "assistant",
            "content": "Old format message 2"
        }
    });

    writeln!(file, "{}", serde_json::to_string(&old_entry1).unwrap()).unwrap();
    writeln!(file, "{}", serde_json::to_string(&old_entry2).unwrap()).unwrap();

    // Read entries - should parse old format correctly
    let entries = session.read_entries().unwrap();
    assert_eq!(entries.len(), 2);

    // Entries should have synthetic IDs
    assert!(entries[0].id.starts_with("synthetic-"));
    assert!(entries[1].id.starts_with("synthetic-"));

    // Parent-child relationship should be established
    assert!(entries[0].parent_id.is_none());
    assert_eq!(entries[1].parent_id, Some(entries[0].id.clone()));

    // Content should be preserved
    assert_eq!(entries[0].content, "Old format message 1");
    assert_eq!(entries[0].role, "user");
    assert_eq!(entries[1].content, "Old format message 2");
    assert_eq!(entries[1].role, "assistant");
}

#[test]
fn session_metadata_serialization() {
    let metadata = SessionMetadata {
        provider: Some("anthropic".into()),
        model: Some("claude-sonnet-4".into()),
        token_count: Some(1500),
        working_directory: Some("/home/user/project".into()),
        reasoning: None,
    };

    let json = serde_json::to_string(&metadata).unwrap();
    let decoded: SessionMetadata = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded.provider, Some("anthropic".into()));
    assert_eq!(decoded.model, Some("claude-sonnet-4".into()));
    assert_eq!(decoded.token_count, Some(1500));
    assert_eq!(decoded.working_directory, Some("/home/user/project".into()));
}

#[test]
fn session_entry_serialization() {
    let entry = SessionEntry {
        id: "test-id".into(),
        parent_id: Some("parent-id".into()),
        timestamp: Utc::now(),
        role: "user".into(),
        content: "Hello".into(),
        metadata: SessionMetadata {
            provider: Some("anthropic".into()),
            model: Some("claude".into()),
            token_count: None,
            working_directory: None,
            reasoning: None,
        },
    };

    let json = serde_json::to_string(&entry).unwrap();
    let decoded: SessionEntry = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded.id, "test-id");
    assert_eq!(decoded.parent_id, Some("parent-id".into()));
    assert_eq!(decoded.role, "user");
    assert_eq!(decoded.content, "Hello");
    assert_eq!(decoded.metadata.model, Some("claude".into()));
}

#[test]
fn append_with_metadata_persists_provider_and_model() {
    let manager = test_manager();
    let session = manager.create_session("test-project", "").unwrap();

    session
        .append_with_metadata(
            &Message::User {
                content: "hello".into(),
            },
            SessionMetadata {
                provider: Some("zhipu-coding-plan".into()),
                model: Some("glm-5.2".into()),
                ..Default::default()
            },
        )
        .unwrap();

    let entries = session.read_entries().unwrap();
    assert_eq!(
        entries[0].metadata.provider,
        Some("zhipu-coding-plan".into())
    );
    assert_eq!(entries[0].metadata.model, Some("glm-5.2".into()));
}

#[test]
fn session_new_has_single_empty_branch() {
    let id = Uuid::new_v4();
    let session = Session::new(
        id,
        "test".into(),
        String::new(),
        PathBuf::from("/tmp/test.jsonl"),
    );

    assert_eq!(session.branches.len(), 1);
    assert_eq!(session.list_branches().len(), 1);

    let branch = session.get_branch(&session.current_branch).unwrap();
    assert!(branch.entries.is_empty());
}

#[test]
fn fork_from_nonexistent_entry_returns_error() {
    let manager = test_manager();
    let mut session = manager.create_session("test-project", "").unwrap();

    let result = session.fork("nonexistent-id");
    assert!(result.is_err());
    match result.unwrap_err() {
        SessionError::EntryNotFound(id) => assert_eq!(id, "nonexistent-id"),
        other => panic!("expected EntryNotFound, got {other:?}"),
    }
}

#[test]
fn list_sessions_scans_directory_correctly() {
    let manager = test_manager();

    // Create sessions in different projects
    let s1 = manager.create_session("project-alpha", "").unwrap();
    let s2 = manager.create_session("project-beta", "").unwrap();

    s1.append(&Message::User {
        content: "First message in alpha".into(),
    })
    .unwrap();

    s2.append(&Message::User {
        content: "First message in beta".into(),
    })
    .unwrap();
    s2.append(&Message::Assistant {
        content: "Reply in beta".into(),
        tool_calls: vec![],
        reasoning: None,
    })
    .unwrap();

    let sessions = manager.list_sessions().unwrap();
    assert_eq!(sessions.len(), 2);

    // Verify both sessions are found
    let alpha = sessions.iter().find(|s| s.id == s1.id).unwrap();
    let beta = sessions.iter().find(|s| s.id == s2.id).unwrap();

    assert_eq!(alpha.message_count, 1);
    assert_eq!(beta.message_count, 2);

    // Verify previews
    assert!(
        alpha
            .last_message_preview
            .contains("First message in alpha")
    );
    assert!(beta.last_message_preview.contains("Reply in beta"));
}

#[test]
fn fork_from_specific_entry_includes_correct_history() {
    let manager = test_manager();
    let mut session = manager.create_session("test-project", "").unwrap();

    session
        .append(&Message::User {
            content: "msg1".into(),
        })
        .unwrap();
    session
        .append(&Message::Assistant {
            content: "reply1".into(),
            tool_calls: vec![],
            reasoning: None,
        })
        .unwrap();
    session
        .append(&Message::User {
            content: "msg2".into(),
        })
        .unwrap();
    session
        .append(&Message::Assistant {
            content: "reply2".into(),
            tool_calls: vec![],
            reasoning: None,
        })
        .unwrap();

    let entries = session.read_entries().unwrap();
    assert_eq!(entries.len(), 4);

    let fork_from_id = entries[1].id.clone();
    let new_branch_id = session.fork(&fork_from_id).unwrap();

    let new_branch = session.get_branch(&new_branch_id).unwrap();
    assert_eq!(new_branch.entries.len(), 2);
    assert_eq!(new_branch.entries[0].content, "msg1");
    assert_eq!(new_branch.entries[1].content, "reply1");
}

#[test]
fn fork_from_current_position_includes_all_entries() {
    let manager = test_manager();
    let mut session = manager.create_session("test-project", "").unwrap();

    session
        .append(&Message::User {
            content: "only message".into(),
        })
        .unwrap();

    let entries = session.read_entries().unwrap();
    let last_entry_id = entries.last().unwrap().id.clone();

    let new_branch_id = session.fork(&last_entry_id).unwrap();
    let new_branch = session.get_branch(&new_branch_id).unwrap();

    assert_eq!(new_branch.entries.len(), 1);
    assert_eq!(new_branch.entries[0].content, "only message");
}

#[test]
fn forked_session_branch_has_correct_root_id() {
    let manager = test_manager();
    let mut session = manager.create_session("test-project", "").unwrap();

    session
        .append(&Message::User {
            content: "root".into(),
        })
        .unwrap();
    session
        .append(&Message::Assistant {
            content: "child".into(),
            tool_calls: vec![],
            reasoning: None,
        })
        .unwrap();

    let entries = session.read_entries().unwrap();
    let fork_point = entries[0].id.clone();

    let new_branch_id = session.fork(&fork_point).unwrap();
    let new_branch = session.get_branch(&new_branch_id).unwrap();

    assert_eq!(new_branch.root_id, fork_point);
}

#[test]
fn arch_s5_update_index_reflects_new_session_in_list_recent() {
    let manager = test_manager();
    let session = manager
        .create_session("arch-s5-list", "")
        .expect("create_session");
    session
        .append(&Message::User {
            content: "hello arch s5".into(),
        })
        .unwrap();

    manager
        .update_index(&session)
        .expect("update_index should succeed");

    let listed = manager
        .list_recent(10)
        .expect("list_recent should succeed after index refresh");
    assert!(
        listed.iter().any(|s| s.id == session.id),
        "list_recent should surface the session after update_index; got {listed:?}"
    );
}

#[test]
fn arch_s5_update_index_reflects_new_session_in_search() {
    let manager = test_manager();
    let session = manager
        .create_session("arch-s5-search", "")
        .expect("create_session");
    session
        .append(&Message::User {
            content: "searchable content alpha".into(),
        })
        .unwrap();

    manager.update_index(&session).expect("update_index");

    let hits = manager
        .search("alpha", 10)
        .expect("search should succeed after index refresh");
    let expected_id = session.id.to_string();
    assert!(
        hits.iter().any(|h| h.session_id == expected_id),
        "FTS5 search should find the session after update_index; got {hits:?}"
    );
}

#[test]
fn arch_s6_fork_identity_sets_new_id_and_path() {
    let mut session = make_session_with_two_entries();
    let original_id = session.id;
    let original_path = session.file_path.clone();

    let new_id = Uuid::new_v4();
    let new_path = original_path
        .parent()
        .unwrap()
        .join(format!("{new_id}.jsonl"));
    let new_branch = Uuid::new_v4().to_string();

    session.with_fork_identity(new_id, new_path.clone(), new_branch.clone());

    assert_eq!(session.id, new_id, "id should be re-stamped to fork UUID");
    assert_ne!(
        session.id, original_id,
        "fork id must differ from source id"
    );
    assert_eq!(
        session.file_path, new_path,
        "file_path should point at the new JSONL"
    );
    assert_eq!(
        session.current_branch, new_branch,
        "current_branch should be the fork's branch id"
    );
}

#[test]
fn arch_s6_fork_index_uses_new_identity() {
    let dir = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(dir.path().to_path_buf());
    let mut source = manager.create_session("arch-s6-index", "").unwrap();
    source
        .append(&Message::User {
            content: "source entry".into(),
        })
        .unwrap();
    let source_id = source.id;
    let entries = source.read_entries().unwrap();
    let fork_point = entries.last().unwrap().id.clone();
    let branch_id = source.fork(&fork_point).unwrap();
    let fork_id = Uuid::new_v4();
    let fork_path = dir
        .path()
        .join("arch-s6-index")
        .join(format!("{fork_id}.jsonl"));
    std::fs::create_dir_all(fork_path.parent().unwrap()).unwrap();
    std::fs::write(&fork_path, b"").unwrap();
    source.with_fork_identity(fork_id, fork_path, branch_id);

    manager.update_index(&source).expect("index fork");
    let recent = manager.list_recent(10).expect("list_recent");
    let by_id: std::collections::HashSet<Uuid> = recent.iter().map(|s| s.id).collect();
    assert!(
        by_id.contains(&fork_id),
        "list_recent should contain fork id {fork_id}; got {by_id:?}"
    );
    assert!(
        !by_id.contains(&source_id),
        "list_recent should NOT contain the source id {source_id} under the fork key; got {by_id:?}"
    );
}

#[test]
fn arch_s6_fork_file_receives_subsequent_appends() {
    let dir = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(dir.path().to_path_buf());
    let mut session = manager.create_session("arch-s6-append", "").unwrap();
    session
        .append(&Message::User {
            content: "before fork".into(),
        })
        .unwrap();
    let entries = session.read_entries().unwrap();
    let fork_point = entries.last().unwrap().id.clone();
    let branch_id = session.fork(&fork_point).unwrap();
    let fork_id = Uuid::new_v4();
    let fork_path = dir
        .path()
        .join("arch-s6-append")
        .join(format!("{fork_id}.jsonl"));
    std::fs::create_dir_all(fork_path.parent().unwrap()).unwrap();
    std::fs::write(&fork_path, b"").unwrap();
    session.with_fork_identity(fork_id, fork_path.clone(), branch_id);

    session
        .append(&Message::Assistant {
            content: "after fork".into(),
            tool_calls: vec![],
            reasoning: None,
        })
        .expect("append should write to fork file");

    let fork_contents = std::fs::read_to_string(&fork_path).expect("read fork file");
    assert!(
        fork_contents.contains("after fork"),
        "fork file should contain the new entry; got {fork_contents:?}"
    );
}

fn make_session_with_two_entries() -> Session {
    let dir = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(dir.path().to_path_buf());
    let session = manager
        .create_session("arch-s6-identity", "")
        .expect("create_session");
    session
        .append(&Message::User {
            content: "first".into(),
        })
        .unwrap();
    session
        .append(&Message::Assistant {
            content: "second".into(),
            tool_calls: vec![],
            reasoning: None,
        })
        .unwrap();
    session
}

#[test]
fn fork_durable_history_clone_source_bytes_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let manager = SessionManager::with_dir(dir.path().to_path_buf());
    let mut source = manager.create_session("fork-clone-test", "").unwrap();
    source
        .append(&Message::User {
            content: "hello".into(),
        })
        .unwrap();
    source
        .append(&Message::Assistant {
            content: "world".into(),
            tool_calls: vec![],
            reasoning: None,
        })
        .unwrap();

    let source_bytes_before = std::fs::read(&source.file_path).unwrap();

    let child_id = Uuid::new_v4();
    let child_path = dir
        .path()
        .join("fork-clone-test")
        .join(format!("{child_id}.jsonl"));
    std::fs::create_dir_all(child_path.parent().unwrap()).unwrap();
    std::fs::write(&child_path, &source_bytes_before).unwrap();

    let source_bytes_after = std::fs::read(&source.file_path).unwrap();
    assert_eq!(
        source_bytes_before, source_bytes_after,
        "source session file must be byte-for-byte unchanged after fork clone"
    );

    let child_bytes = std::fs::read(&child_path).unwrap();
    assert_eq!(
        source_bytes_before, child_bytes,
        "child session file must be an exact copy of source"
    );

    let mut child = Session::new(child_id, "fork-clone-test".into(), "".into(), child_path);
    let child_messages = child.read_messages().unwrap();
    assert_eq!(
        child_messages.len(),
        2,
        "child should have same message count"
    );
    assert_ne!(child.id, source.id, "child must have distinct session id");
}

#[test]
fn delete_session_removes_file_and_index_entry() {
    let manager = test_manager();
    let session = manager
        .create_session("delete-test", "delete-test")
        .unwrap();
    session
        .append(&Message::User {
            content: "hello".into(),
        })
        .unwrap();

    let id = session.id;
    assert!(session.file_path.exists(), "precondition: file exists");

    manager.delete_session(&id).expect("delete should succeed");

    assert!(
        !session.file_path.exists(),
        "session file must be removed from disk"
    );
    let sessions = manager.list_workspace_sessions("delete-test").unwrap();
    assert!(
        sessions.iter().all(|s| s.id != id),
        "deleted session must not appear in workspace listing"
    );
}

#[test]
fn cleanup_candidates_respect_max_sessions_and_protected_ids() {
    let manager = test_manager();
    let workspace = "cleanup-protect";
    let old = manager.create_session("cleanup", workspace).unwrap();
    old.append(&Message::User {
        content: "old".into(),
    })
    .unwrap();
    let protected = manager.create_session("cleanup", workspace).unwrap();
    protected
        .append(&Message::User {
            content: "protected".into(),
        })
        .unwrap();
    let newest = manager.create_session("cleanup", workspace).unwrap();
    newest
        .append(&Message::User {
            content: "newest".into(),
        })
        .unwrap();

    let policy = crate::SessionCleanupPolicy {
        workspace_root: Some(workspace.to_string()),
        max_sessions_per_workspace: Some(1),
        max_age_days: None,
        protected_session_ids: vec![protected.id],
    };

    let candidates = manager.cleanup_candidates(&policy).unwrap();
    assert_eq!(
        candidates.len(),
        1,
        "one unprotected session should exceed the per-workspace retention limit"
    );
    assert!(
        candidates
            .iter()
            .all(|candidate| candidate.workspace_root == workspace),
        "cleanup should stay scoped to the requested workspace"
    );
    assert!(
        candidates
            .iter()
            .all(|candidate| candidate.id == old.id || candidate.id == newest.id),
        "only unprotected sessions may be selected"
    );
    assert!(
        !candidates
            .iter()
            .any(|candidate| candidate.id == protected.id),
        "protected session must never be selected"
    );
}

#[test]
fn apply_cleanup_removes_file_and_index_entry() {
    let manager = test_manager();
    let workspace = "cleanup-apply";
    let stale = manager.create_session("cleanup", workspace).unwrap();
    stale
        .append(&Message::User {
            content: "stale indexed content".into(),
        })
        .unwrap();
    let keep = manager.create_session("cleanup", workspace).unwrap();
    keep.append(&Message::User {
        content: "keep indexed content".into(),
    })
    .unwrap();
    manager.update_index(&stale).unwrap();
    manager.update_index(&keep).unwrap();

    let policy = crate::SessionCleanupPolicy {
        workspace_root: Some(workspace.to_string()),
        max_sessions_per_workspace: Some(0),
        max_age_days: None,
        protected_session_ids: vec![keep.id],
    };

    let report = manager.apply_cleanup(&policy).unwrap();
    assert_eq!(report.removed, 1);
    assert!(report.bytes_removed > 0);
    assert!(
        !stale.file_path.exists(),
        "cleanup must remove selected JSONL file"
    );
    assert!(
        keep.file_path.exists(),
        "protected retained file must remain"
    );
    assert!(
        manager
            .search("stale", 10)
            .unwrap()
            .iter()
            .all(|result| result.session_id != stale.id.to_string()),
        "cleanup must remove deleted session rows from the index"
    );
}

#[test]
fn session_index_maintenance_operations_run() {
    let manager = test_manager();
    let session = manager
        .create_session("maintenance", "maintenance-workspace")
        .unwrap();
    session
        .append(&Message::User {
            content: "maintenance indexed content".into(),
        })
        .unwrap();
    manager.update_index(&session).unwrap();

    manager.checkpoint_index().unwrap();
    manager.vacuum_index().unwrap();
    let results = manager.search("maintenance", 10).unwrap();
    assert!(
        results
            .iter()
            .any(|result| result.session_id == session.id.to_string()),
        "maintenance must not remove indexed data"
    );
}

#[test]
fn reconcile_index_repairs_stale_entries() {
    let manager = test_manager();
    let session = manager
        .create_session("reconcile-test", "reconcile-test")
        .unwrap();
    session
        .append(&Message::User {
            content: "first".into(),
        })
        .unwrap();
    session
        .append(&Message::Assistant {
            content: "hi".into(),
            tool_calls: vec![],
            reasoning: None,
        })
        .unwrap();

    let fixed = manager.reconcile_index().expect("reconcile should succeed");
    assert!(fixed >= 1, "reconcile should reindex at least one entry");
}

#[test]
fn find_session_file_missing_returns_error() {
    use crate::SessionError;
    let manager = test_manager();
    let result = manager.delete_session(&Uuid::new_v4());
    match result {
        Err(SessionError::SessionNotFound(_)) => {}
        other => panic!("expected SessionNotFound, got {other:?}"),
    }
}

#[test]
fn snapshot_bytes_returns_file_contents() {
    let manager = test_manager();
    let session = manager
        .create_session("snapshot-test", "snapshot-test")
        .unwrap();
    session
        .append(&Message::User {
            content: "first".into(),
        })
        .unwrap();
    session
        .append(&Message::Assistant {
            content: "second".into(),
            tool_calls: vec![],
            reasoning: None,
        })
        .unwrap();

    let bytes = session.snapshot_bytes().expect("snapshot should succeed");
    let on_disk = std::fs::read(&session.file_path).unwrap();
    assert_eq!(bytes, on_disk, "snapshot must match disk bytes exactly");
    assert!(
        bytes.windows(b"first".len()).any(|w| w == b"first"),
        "snapshot must contain first message"
    );
    assert!(
        bytes.windows(b"second".len()).any(|w| w == b"second"),
        "snapshot must contain second message"
    );
}

#[test]
fn snapshot_bytes_missing_file_returns_error() {
    let manager = test_manager();
    let session = manager
        .create_session("snapshot-missing", "snapshot-missing")
        .unwrap();
    std::fs::remove_file(&session.file_path).unwrap();

    let result = session.snapshot_bytes();
    assert!(result.is_err(), "missing file must error, not panic");
}
