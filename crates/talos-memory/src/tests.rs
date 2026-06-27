use super::*;
use chrono::Utc;
use rusqlite::params;

use crate::prompt::is_hidden_output;

fn make_item(id: &str, key: &str, content: &str) -> MemoryItem {
    let now = Utc::now();
    MemoryItem {
        id: id.to_string(),
        kind: MemoryKind::Semantic,
        key: key.to_string(),
        content: content.to_string(),
        confidence: 0.8,
        created_at: now,
        last_reinforced: now,
        last_accessed: None,
        contradiction_ref: None,
    }
}

#[test]
fn test_schema_migration_creates_tables() {
    let store = MemoryStore::open_memory().unwrap();

    let table_count: i64 = store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type IN ('table', 'view') \
                 AND name IN ('memory_items', 'evidence_links', 'schema_version')",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(table_count, 3, "All three tables should exist");

    let fts_count: i64 = store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='memory_fts'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(fts_count, 1, "FTS5 virtual table should exist");
}

#[test]
fn test_insert_and_retrieve() {
    let mut store = MemoryStore::open_memory().unwrap();
    let content = "Rust is a systems programming language focused on safety".to_string();
    let item = make_item("mem-1", "rust", &content);
    store.insert(item).unwrap();

    let results = store
        .retrieve("Rust systems programming safety", 10)
        .unwrap();
    assert!(!results.is_empty(), "Should find the inserted item");
    assert_eq!(results[0].item.content, content);
}

#[test]
fn test_add_only_preserves_conflicts() {
    let mut store = MemoryStore::open_memory().unwrap();

    let item1 = make_item("mem-1", "language", "Python is dynamically typed");
    let item2 = make_item("mem-2", "language", "Python is statically typed");

    store.insert(item1).unwrap();
    store.insert(item2).unwrap();

    assert_eq!(
        store.count().unwrap(),
        2,
        "Both conflicting items should exist"
    );
}

#[test]
fn test_exact_dedup_prevents_duplicates() {
    let mut store = MemoryStore::open_memory().unwrap();

    let item = make_item("mem-1", "fact", "The sky is blue");
    assert!(store.insert(item).unwrap(), "First insert should succeed");

    let item_dup = make_item("mem-2", "fact", "The sky is blue");
    assert!(
        !store.insert(item_dup).unwrap(),
        "Duplicate should be ignored"
    );

    assert_eq!(store.count().unwrap(), 1, "Only one row should exist");
}

#[test]
fn test_bounded_retrieval_respects_limit() {
    let mut store = MemoryStore::open_memory().unwrap();

    for i in 0..5 {
        let item = make_item(
            &format!("mem-{i}"),
            "topic",
            &format!("Item number {i} about testing retrieval limits"),
        );
        store.insert(item).unwrap();
    }

    let results = store.retrieve("testing retrieval", 3).unwrap();
    assert!(
        results.len() <= 3,
        "Should respect limit of 3, got {}",
        results.len()
    );
}

#[test]
fn test_retrieval_includes_evidence() {
    let mut store = MemoryStore::open_memory().unwrap();

    let item = make_item(
        "mem-1",
        "evidence-test",
        "Evidence links are important for provenance",
    );
    store.insert(item).unwrap();

    let link = EvidenceLink {
        id: "ev-1".to_string(),
        memory_id: "mem-1".to_string(),
        source_type: "session".to_string(),
        source_ref: "session-abc".to_string(),
        created_at: Utc::now(),
    };
    store.insert_evidence(link).unwrap();

    let results = store.retrieve("evidence provenance", 10).unwrap();
    assert!(!results.is_empty());
    assert!(
        !results[0].evidence.is_empty(),
        "Should include evidence links"
    );
    assert_eq!(results[0].evidence[0].source_ref, "session-abc");
}

#[test]
fn test_retrieval_scoring_is_deterministic() {
    let mut store = MemoryStore::open_memory().unwrap();

    let item = make_item(
        "mem-1",
        "deterministic",
        "Deterministic scoring test content for verification",
    );
    store.insert(item).unwrap();

    let results1 = store
        .retrieve("deterministic scoring verification", 10)
        .unwrap();
    let results2 = store
        .retrieve("deterministic scoring verification", 10)
        .unwrap();

    assert_eq!(results1.len(), results2.len());
    if !results1.is_empty() {
        assert!(
            (results1[0].score - results2[0].score).abs() < 0.01,
            "Scores should be nearly identical: {} vs {}",
            results1[0].score,
            results2[0].score
        );
    }
}

#[test]
fn test_retrieve_empty_query_returns_nothing() {
    let mut store = MemoryStore::open_memory().unwrap();

    let item = make_item("mem-1", "test", "Some content here");
    store.insert(item).unwrap();

    let results = store.retrieve("", 10).unwrap();
    assert!(results.is_empty(), "Empty query should return no results");

    let results = store.retrieve("   ", 10).unwrap();
    assert!(
        results.is_empty(),
        "Whitespace-only query should return no results"
    );
}

#[test]
fn test_get_by_id() {
    let mut store = MemoryStore::open_memory().unwrap();

    let item = make_item("mem-1", "lookup", "Lookup by ID test content");
    store.insert(item).unwrap();

    let found = store.get("mem-1").unwrap();
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.id, "mem-1");
    assert_eq!(found.content, "Lookup by ID test content");
    assert_eq!(found.kind, MemoryKind::Semantic);

    let not_found = store.get("nonexistent").unwrap();
    assert!(not_found.is_none());
}

#[test]
fn test_procedural_kind_roundtrip() {
    let mut store = MemoryStore::open_memory().unwrap();

    let now = Utc::now();
    let item = MemoryItem {
        id: "proc-1".to_string(),
        kind: MemoryKind::Procedural,
        key: "git-workflow".to_string(),
        content: "Always rebase feature branches onto main".to_string(),
        confidence: 0.9,
        created_at: now,
        last_reinforced: now,
        last_accessed: None,
        contradiction_ref: None,
    };
    store.insert(item).unwrap();

    let found = store.get("proc-1").unwrap().unwrap();
    assert_eq!(found.kind, MemoryKind::Procedural);
}

#[test]
fn test_evidence_link_persists() {
    let mut store = MemoryStore::open_memory().unwrap();

    let item = make_item("mem-1", "test", "Test content");
    store.insert(item).unwrap();

    let link = EvidenceLink {
        id: "ev-1".to_string(),
        memory_id: "mem-1".to_string(),
        source_type: "tool_call".to_string(),
        source_ref: "read:src/main.rs".to_string(),
        created_at: Utc::now(),
    };
    store.insert_evidence(link).unwrap();

    let results = store.retrieve("Test content", 10).unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].evidence.len(), 1);
    assert_eq!(results[0].evidence[0].source_type, "tool_call");
}

#[test]
fn test_evidence_requires_existing_memory() {
    let store = MemoryStore::open_memory().unwrap();

    let link = EvidenceLink {
        id: "ev-orphan".to_string(),
        memory_id: "missing-memory".to_string(),
        source_type: "session".to_string(),
        source_ref: "session-missing".to_string(),
        created_at: Utc::now(),
    };

    let err = store
        .insert_evidence(link)
        .expect_err("foreign-key enforcement must reject orphan evidence");
    assert!(
        err.to_string().contains("FOREIGN KEY") || err.to_string().contains("constraint failed"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_memory_maintenance_operations_run() {
    let mut store = MemoryStore::open_memory().unwrap();
    store
        .insert(make_item(
            "mem-1",
            "maintenance",
            "maintenance test content",
        ))
        .unwrap();

    store.checkpoint_truncate().unwrap();
    store.vacuum().unwrap();
    assert_eq!(store.count().unwrap(), 1);
}

#[test]
fn test_count_reflects_inserts() {
    let mut store = MemoryStore::open_memory().unwrap();
    assert_eq!(store.count().unwrap(), 0);

    store.insert(make_item("m1", "k", "c1")).unwrap();
    assert_eq!(store.count().unwrap(), 1);

    store.insert(make_item("m2", "k", "c2")).unwrap();
    assert_eq!(store.count().unwrap(), 2);

    // Exact dup should not increase count.
    store.insert(make_item("m3", "k", "c1")).unwrap();
    assert_eq!(store.count().unwrap(), 2);
}

// --- format_memory_prompt tests ---

#[test]
fn format_memory_prompt_disabled_returns_none() {
    let mut store = MemoryStore::open_memory().unwrap();
    store
        .insert(make_item("mem-1", "test", "some content"))
        .unwrap();

    let config = MemoryPromptConfig {
        enabled: false,
        ..Default::default()
    };
    assert!(format_memory_prompt(&store, "test", &config).is_none());
}

#[test]
fn format_memory_prompt_no_results_returns_none() {
    let store = MemoryStore::open_memory().unwrap();

    let config = MemoryPromptConfig {
        enabled: true,
        ..Default::default()
    };
    assert!(format_memory_prompt(&store, "nonexistent query xyz", &config).is_none());
}

#[test]
fn format_memory_prompt_produces_bounded_section() {
    let mut store = MemoryStore::open_memory().unwrap();

    store
        .insert(make_item(
            "mem-1",
            "rust",
            "Rust is a systems language focused on safety",
        ))
        .unwrap();
    store
        .insert(make_item(
            "mem-2",
            "rust",
            "Rust has zero-cost abstractions and no garbage collector",
        ))
        .unwrap();
    store
        .insert(make_item(
            "mem-3",
            "testing",
            "Testing is important for software quality",
        ))
        .unwrap();

    // Add evidence for provenance.
    store
        .insert_evidence(EvidenceLink {
            id: "ev-1".to_string(),
            memory_id: "mem-1".to_string(),
            source_type: "session".to_string(),
            source_ref: "session-abc:entry-1:0".to_string(),
            created_at: Utc::now(),
        })
        .unwrap();

    let config = MemoryPromptConfig {
        enabled: true,
        max_items: 5,
        max_chars: 2000,
    };
    let result = format_memory_prompt(&store, "Rust safety", &config);

    assert!(result.is_some(), "Should produce output");
    let text = result.unwrap();
    assert!(text.contains("## Relevant Memory"), "Should contain header");
    assert!(text.contains("confidence="), "Should contain confidence");
    assert!(text.contains("source:"), "Should contain source reference");
    assert!(text.len() <= config.max_chars, "Should respect max_chars");
}

#[test]
fn format_memory_prompt_truncates_on_budget() {
    let mut store = MemoryStore::open_memory().unwrap();

    let long_content = "This is a very long memory item that contains a lot of text to test the truncation behavior of the format_memory_prompt function when the character budget is exceeded by the accumulated output length of multiple memory items combined together in the final formatted string".repeat(3);
    store
        .insert(make_item("mem-1", "long", &long_content))
        .unwrap();

    let config = MemoryPromptConfig {
        enabled: true,
        max_items: 5,
        max_chars: 100,
    };
    let result = format_memory_prompt(&store, "long memory", &config);

    assert!(result.is_some(), "Should produce some output");
    let text = result.unwrap();
    assert!(
        text.contains("truncated"),
        "Should contain truncation notice, got: {text}"
    );
    assert!(text.len() <= config.max_chars, "Should respect max_chars");
}

#[test]
fn format_memory_prompt_filters_hidden_output() {
    let mut store = MemoryStore::open_memory().unwrap();

    // Insert a clean memory item.
    store
        .insert(make_item(
            "mem-1",
            "clean",
            "Rust is a safe systems language",
        ))
        .unwrap();

    // Insert items that look like hidden tool output.
    store
        .insert(make_item(
            "mem-2",
            "tool-like",
            "<tool_result>file read successfully</tool_result>",
        ))
        .unwrap();
    store
        .insert(make_item(
            "mem-3",
            "tool-like-2",
            "Tool output: the file contains 42 lines",
        ))
        .unwrap();
    store
        .insert(make_item(
            "mem-4",
            "tool-like-3",
            "is_error: true, message: something failed",
        ))
        .unwrap();

    let config = MemoryPromptConfig {
        enabled: true,
        max_items: 10,
        max_chars: 4000,
    };
    let result = format_memory_prompt(&store, "tool result error", &config);

    // The clean item may or may not appear depending on FTS scoring.
    // But the hidden-output items must NOT appear.
    if let Some(text) = result {
        assert!(
            !text.contains("<tool_result>"),
            "Should not contain tool result tags"
        );
        assert!(
            !text.contains("Tool output:"),
            "Should not contain tool output prefix"
        );
        assert!(
            !text.contains("is_error:"),
            "Should not contain error markers"
        );
    }
}

#[test]
fn format_memory_prompt_marks_contradictions() {
    let mut store = MemoryStore::open_memory().unwrap();

    let now = Utc::now();
    let item = MemoryItem {
        id: "mem-contradict".to_string(),
        kind: MemoryKind::Semantic,
        key: "conflict".to_string(),
        content: "Python is dynamically typed".to_string(),
        confidence: 0.7,
        created_at: now,
        last_reinforced: now,
        last_accessed: None,
        contradiction_ref: Some("ref-123".to_string()),
    };
    store.insert(item).unwrap();

    let config = MemoryPromptConfig {
        enabled: true,
        max_items: 5,
        max_chars: 2000,
    };
    let result = format_memory_prompt(&store, "Python typed", &config);

    assert!(result.is_some(), "Should produce output");
    let text = result.unwrap();
    assert!(
        text.contains("CONTRADICTION"),
        "Should mark contradiction, got: {text}"
    );
}

// --- Entity extraction tests ---

#[test]
fn extract_file_entities() {
    let content = "Edit src/main.rs and update Cargo.toml for the new feature";
    let entities = extract_entities(content);

    let files: Vec<&str> = entities
        .iter()
        .filter(|(_, k)| *k == EntityKind::File)
        .map(|(n, _)| n.as_str())
        .collect();

    assert!(
        files.contains(&"src/main.rs"),
        "Should find src/main.rs, got: {files:?}"
    );
    assert!(
        files.contains(&"Cargo.toml"),
        "Should find Cargo.toml, got: {files:?}"
    );
}

#[test]
fn extract_url_entities() {
    let content = "See https://docs.rs/talos for details and visit http://example.com/path?q=1";
    let entities = extract_entities(content);

    let urls: Vec<&str> = entities
        .iter()
        .filter(|(_, k)| *k == EntityKind::Url)
        .map(|(n, _)| n.as_str())
        .collect();

    assert!(
        urls.iter().any(|u| u.starts_with("https://docs.rs")),
        "Should find https URL, got: {urls:?}"
    );
    assert!(
        urls.iter().any(|u| u.starts_with("http://example.com")),
        "Should find http URL, got: {urls:?}"
    );
}

#[test]
fn extract_code_entities() {
    let content = "Use MemoryStore and extract_entities for the implementation";
    let entities = extract_entities(content);

    let codes: Vec<&str> = entities
        .iter()
        .filter(|(_, k)| *k == EntityKind::Code)
        .map(|(n, _)| n.as_str())
        .collect();

    assert!(
        codes.contains(&"MemoryStore"),
        "Should find MemoryStore, got: {codes:?}"
    );
    assert!(
        codes.contains(&"extract_entities"),
        "Should find extract_entities, got: {codes:?}"
    );
}

#[test]
fn extract_entities_malformed_input_no_panic() {
    // Empty string.
    let _ = extract_entities("");

    // Very long string.
    let long = "a".repeat(100_000);
    let _ = extract_entities(&long);

    // Binary-like content.
    let binary = "\0\x01\x02\x03\x7f\x7e";
    let _ = extract_entities(binary);

    // Only punctuation.
    let _ = extract_entities("!@#$%^&*()_+-=[]{}|;':\",./<>?");
}

#[test]
fn entity_overlap_boosts_retrieval() {
    let mut store = MemoryStore::open_memory().unwrap();

    // Memory with file paths that match the query.
    let item1 = make_item(
        "mem-entity-1",
        "entity-test",
        "Update src/main.rs to fix the bug in Cargo.toml",
    );
    store.insert(item1).unwrap();

    // Memory with unrelated content.
    let item2 = make_item(
        "mem-entity-2",
        "entity-test",
        "The weather is nice today and the sky is blue",
    );
    store.insert(item2).unwrap();

    let results = store.retrieve("src/main.rs Cargo.toml", 10).unwrap();
    assert!(!results.is_empty(), "Should find results");

    // The entity-matching item should rank higher or at least appear.
    let entity_1_pos = results.iter().position(|r| r.item.id == "mem-entity-1");
    let entity_2_pos = results.iter().position(|r| r.item.id == "mem-entity-2");

    if let (Some(p1), Some(p2)) = (entity_1_pos, entity_2_pos) {
        assert!(
            p1 < p2,
            "Entity-matching item should rank higher: pos1={p1}, pos2={p2}"
        );
    }
}

#[test]
fn procedural_memory_storage_and_retrieval() {
    let mut store = MemoryStore::open_memory().unwrap();

    let now = Utc::now();
    let item = MemoryItem {
        id: "proc-test-1".to_string(),
        kind: MemoryKind::Procedural,
        key: "commit-workflow".to_string(),
        content: "Always run cargo fmt before you commit code to the repository".to_string(),
        confidence: 0.9,
        created_at: now,
        last_reinforced: now,
        last_accessed: None,
        contradiction_ref: None,
    };
    store.insert(item).unwrap();

    let results = store.retrieve("cargo fmt commit", 10).unwrap();
    assert!(!results.is_empty(), "Should retrieve procedural memory");

    let found = results.iter().find(|r| r.item.id == "proc-test-1");
    assert!(found.is_some(), "Should find the procedural item");
    assert_eq!(found.unwrap().item.kind, MemoryKind::Procedural);
}

#[test]
fn procedural_memory_has_no_permission_authority() {
    let mut store = MemoryStore::open_memory().unwrap();

    let item = MemoryItem {
        id: "proc-perm-test".to_string(),
        kind: MemoryKind::Procedural,
        key: "commit-workflow".to_string(),
        content: "Always run cargo fmt before committing code".to_string(),
        confidence: 0.9,
        created_at: Utc::now(),
        last_reinforced: Utc::now(),
        last_accessed: None,
        contradiction_ref: None,
    };
    assert!(store.insert(item).unwrap());

    let results = store.retrieve("cargo fmt", 10).unwrap();
    assert!(!results.is_empty());

    // Memory retrieval returns data only — no permission grant.
    // MemoryStore has no methods that grant, approve, or bypass permissions.
}

#[test]
fn entity_linking_on_insert() {
    let mut store = MemoryStore::open_memory().unwrap();

    let item = make_item(
        "mem-link-test",
        "entity-linking",
        "Update src/lib.rs and check Cargo.toml for dependencies",
    );
    store.insert(item).unwrap();

    // Verify entities were linked in the database.
    let entity_count: i64 = store
        .conn
        .query_row(
            "SELECT COUNT(*) FROM memory_entities WHERE memory_id = ?1",
            params!["mem-link-test"],
            |row| row.get(0),
        )
        .unwrap();

    assert!(
        entity_count > 0,
        "Should have linked entities, got count={entity_count}"
    );

    // Verify the entities table has entries.
    let total_entities: i64 = store
        .conn
        .query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))
        .unwrap();

    assert!(
        total_entities > 0,
        "Should have entities in the table, got count={total_entities}"
    );
}

#[test]
fn corrupt_db_degrades_gracefully() {
    let dir = tempfile::tempdir().unwrap();
    let corrupt_path = dir.path().join("corrupt.db");
    std::fs::write(&corrupt_path, b"this is not a valid sqlite database file").unwrap();

    let result = MemoryStore::open(&corrupt_path);
    assert!(
        result.is_err(),
        "Opening a corrupt DB should return an error, not panic"
    );
    let err = match result {
        Err(e) => e,
        Ok(_) => panic!("expected error for corrupt DB"),
    };
    let err_msg = err.to_string().to_lowercase();
    assert!(
        err_msg.contains("database")
            || err_msg.contains("file")
            || err_msg.contains("malformed")
            || err_msg.contains("not a database"),
        "Error should be actionable, got: {err}"
    );
}

#[test]
fn missing_db_path_handled() {
    let dir = tempfile::tempdir().unwrap();
    let nested_path = dir.path().join("nonexistent").join("sub").join("memory.db");

    let result = MemoryStore::open(&nested_path);
    assert!(
        result.is_ok(),
        "Opening a DB in a nonexistent parent should create the path"
    );
}

#[test]
fn memory_status_reports_counts() {
    let mut store = MemoryStore::open_memory().unwrap();

    for i in 0..3 {
        let item = make_item(
            &format!("sem-{i}"),
            "status-test",
            &format!("Update src/lib.rs for semantic fact {i}"),
        );
        store.insert(item).unwrap();
    }

    let now = Utc::now();
    for i in 0..2 {
        let item = MemoryItem {
            id: format!("proc-{i}"),
            kind: MemoryKind::Procedural,
            key: "status-proc".to_string(),
            content: format!("Run Cargo.toml test step {i}"),
            confidence: 0.9,
            created_at: now,
            last_reinforced: now,
            last_accessed: None,
            contradiction_ref: None,
        };
        store.insert(item).unwrap();
    }

    for i in 0..3 {
        store
            .insert_evidence(EvidenceLink {
                id: format!("ev-{i}"),
                memory_id: format!("sem-{i}"),
                source_type: "session".to_string(),
                source_ref: format!("session-{i}"),
                created_at: Utc::now(),
            })
            .unwrap();
    }

    let status = store.memory_status().unwrap();
    assert_eq!(status.total_items, 5);
    assert_eq!(status.semantic_count, 3);
    assert_eq!(status.procedural_count, 2);
    assert_eq!(status.evidence_count, 3);
    assert!(status.entity_count > 0, "entity_count should be > 0");
    assert!(status.db_path.is_none());
    assert_eq!(status.db_size_bytes, 0);
}

#[test]
fn retention_dry_run_no_deletion() {
    let mut store = MemoryStore::open_memory().unwrap();
    let now = Utc::now();

    let low_conf = MemoryItem {
        id: "low-conf".to_string(),
        kind: MemoryKind::Semantic,
        key: "low-confidence".to_string(),
        content: "A low confidence memory".to_string(),
        confidence: 0.2,
        created_at: now,
        last_reinforced: now,
        last_accessed: None,
        contradiction_ref: None,
    };
    store.insert(low_conf).unwrap();

    let high_conf = MemoryItem {
        id: "high-conf".to_string(),
        kind: MemoryKind::Semantic,
        key: "high-confidence".to_string(),
        content: "A high confidence memory".to_string(),
        confidence: 0.9,
        created_at: now,
        last_reinforced: now,
        last_accessed: None,
        contradiction_ref: None,
    };
    store.insert(high_conf).unwrap();

    let policy = RetentionPolicy {
        min_confidence: Some(0.5),
        ..Default::default()
    };

    let candidates = store.retention_candidates(&policy).unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].id, "low-conf");

    let count_before = store.count().unwrap();
    assert_eq!(count_before, 2);
}

#[test]
fn retention_key_preview_truncated() {
    let mut store = MemoryStore::open_memory().unwrap();
    let now = Utc::now();

    let long_key =
        "this_is_a_very_long_key_that_should_be_truncated_in_the_retention_candidate_output"
            .to_string();
    let item = MemoryItem {
        id: "long-key".to_string(),
        kind: MemoryKind::Semantic,
        key: long_key.clone(),
        content: "Some content".to_string(),
        confidence: 0.1,
        created_at: now,
        last_reinforced: now,
        last_accessed: None,
        contradiction_ref: None,
    };
    store.insert(item).unwrap();

    let policy = RetentionPolicy {
        min_confidence: Some(0.5),
        ..Default::default()
    };

    let candidates = store.retention_candidates(&policy).unwrap();
    assert_eq!(candidates.len(), 1);
    assert!(
        candidates[0].key_preview.len() <= 30,
        "key_preview should be <= 30 chars, got {} chars: '{}'",
        candidates[0].key_preview.len(),
        candidates[0].key_preview
    );
    assert!(candidates[0].key_preview.ends_with("..."));
}

#[test]
fn retention_unreinforced_only() {
    let mut store = MemoryStore::open_memory().unwrap();
    let now = Utc::now();

    let with_evidence = MemoryItem {
        id: "with-ev".to_string(),
        kind: MemoryKind::Semantic,
        key: "reinforced".to_string(),
        content: "Has evidence".to_string(),
        confidence: 0.3,
        created_at: now,
        last_reinforced: now,
        last_accessed: None,
        contradiction_ref: None,
    };
    store.insert(with_evidence).unwrap();
    store
        .insert_evidence(EvidenceLink {
            id: "ev-1".to_string(),
            memory_id: "with-ev".to_string(),
            source_type: "session".to_string(),
            source_ref: "session-1".to_string(),
            created_at: now,
        })
        .unwrap();

    let without_evidence = MemoryItem {
        id: "without-ev".to_string(),
        kind: MemoryKind::Semantic,
        key: "unreinforced".to_string(),
        content: "No evidence".to_string(),
        confidence: 0.3,
        created_at: now,
        last_reinforced: now,
        last_accessed: None,
        contradiction_ref: None,
    };
    store.insert(without_evidence).unwrap();

    let policy = RetentionPolicy {
        unreinforced_only: true,
        ..Default::default()
    };

    let candidates = store.retention_candidates(&policy).unwrap();
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].id, "without-ev");
}

#[test]
fn end_to_end_memory_pipeline() {
    let mut store = MemoryStore::open_memory().unwrap();
    let now = Utc::now();

    let semantic = MemoryItem {
        id: "e2e-sem".to_string(),
        kind: MemoryKind::Semantic,
        key: "rust-safety".to_string(),
        content: "Rust guarantees memory safety without a garbage collector".to_string(),
        confidence: 0.85,
        created_at: now,
        last_reinforced: now,
        last_accessed: None,
        contradiction_ref: None,
    };
    store.insert(semantic).unwrap();

    store
        .insert_evidence(EvidenceLink {
            id: "e2e-ev".to_string(),
            memory_id: "e2e-sem".to_string(),
            source_type: "session".to_string(),
            source_ref: "session-e2e:turn-0".to_string(),
            created_at: now,
        })
        .unwrap();

    let procedural = MemoryItem {
        id: "e2e-proc".to_string(),
        kind: MemoryKind::Procedural,
        key: "cargo-test".to_string(),
        content: "Run cargo test before merging".to_string(),
        confidence: 0.95,
        created_at: now,
        last_reinforced: now,
        last_accessed: None,
        contradiction_ref: None,
    };
    store.insert(procedural).unwrap();

    let sem_results = store.retrieve("Rust memory safety", 5).unwrap();
    assert!(
        sem_results.iter().any(|r| r.item.id == "e2e-sem"),
        "Should retrieve semantic memory"
    );

    let proc_results = store.retrieve("cargo test", 5).unwrap();
    assert!(
        proc_results.iter().any(|r| r.item.id == "e2e-proc"),
        "Should retrieve procedural memory"
    );

    let config = MemoryPromptConfig {
        enabled: true,
        max_items: 5,
        max_chars: 2000,
    };
    let prompt = format_memory_prompt(&store, "Rust memory safety", &config);
    assert!(prompt.is_some(), "Should produce formatted prompt");
    let prompt_text = prompt.unwrap();
    assert!(
        prompt_text.contains("memory safety"),
        "Prompt should contain memory content"
    );

    let status = store.memory_status().unwrap();
    assert_eq!(status.total_items, 2);
    assert_eq!(status.semantic_count, 1);
    assert_eq!(status.procedural_count, 1);
    assert_eq!(status.evidence_count, 1);
}

#[test]
fn hidden_output_blocks_json_tool_result_marker() {
    assert!(is_hidden_output(
        r#"{"type": "tool_result", "content": "x"}"#
    ));
    assert!(is_hidden_output(r#"{"type":"tool_result"}"#));
}

#[test]
fn hidden_output_blocks_json_role_tool_marker() {
    assert!(is_hidden_output(r#"{"role": "tool", "content": "x"}"#));
    assert!(is_hidden_output(r#"{"role":"tool"}"#));
}

#[test]
fn hidden_output_blocks_whitespace_padded_tag() {
    assert!(is_hidden_output("< tool_result >"));
    assert!(is_hidden_output("<tool_result  >"));
    assert!(is_hidden_output("  <tool_result>  "));
}

#[test]
fn hidden_output_blocks_system_reminder() {
    assert!(is_hidden_output(
        "<system-reminder>check your work</system-reminder>"
    ));
    assert!(is_hidden_output("system-reminder: verify output"));
    assert!(is_hidden_output("<system>hidden</system>"));
}

#[test]
fn hidden_output_blocks_anthropic_tool_use() {
    assert!(is_hidden_output("tool_use id=tu_123"));
    assert!(is_hidden_output("tool_use_id: tu_123"));
    assert!(is_hidden_output("function_call: read_file"));
}

#[test]
fn hidden_output_allows_normal_text() {
    assert!(!is_hidden_output("The project uses Rust for safety."));
    assert!(!is_hidden_output("Always run cargo test before merging."));
    assert!(!is_hidden_output("User prefers dark mode in the terminal."));
}

#[test]
fn hidden_output_case_insensitive() {
    assert!(is_hidden_output("<TOOL_RESULT>"));
    assert!(is_hidden_output("Tool_Result"));
    assert!(is_hidden_output("TOOL_CALL"));
    assert!(is_hidden_output("TOOL_USE"));
}
