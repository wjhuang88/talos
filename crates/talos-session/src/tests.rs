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
        }
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
    };

    session.append(&msg).unwrap();

    let messages = session.read_messages().unwrap();
    assert_eq!(messages.len(), 1);
    match &messages[0] {
        Message::Assistant {
            content,
            tool_calls,
        } => {
            assert_eq!(content, "Let me check that file.");
            assert_eq!(tool_calls.len(), 1);
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
        model: Some("claude-sonnet-4".into()),
        token_count: Some(1500),
        working_directory: Some("/home/user/project".into()),
    };

    let json = serde_json::to_string(&metadata).unwrap();
    let decoded: SessionMetadata = serde_json::from_str(&json).unwrap();

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
            model: Some("claude".into()),
            token_count: None,
            working_directory: None,
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
