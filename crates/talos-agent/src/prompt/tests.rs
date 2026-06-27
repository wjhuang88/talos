use super::*;
use talos_skill::SkillIndex;

// --- Basic assembly tests ---

#[test]
fn test_build_with_default_identity() {
    let builder = SystemPromptBuilder::new();
    let prompt = builder.build();

    assert!(prompt.contains("# Identity"));
    assert!(prompt.contains("You are Talos"));
}

#[test]
fn test_build_with_all_components() {
    let builder = SystemPromptBuilder::new()
        .with_tools(vec![ToolDescription {
            name: "read".into(),
            description: "Read a file".into(),
            ..Default::default()
        }])
        .with_skill_index(vec![SkillIndex {
            name: "git-skill".into(),
            description: "Git operations".into(),
            triggers: vec!["git".into()],
            estimated_tokens: 0,
        }])
        .with_context_files(vec![ContextFile {
            path: "AGENTS.md".into(),
            content: "# Project Rules\nBe helpful.".into(),
        }])
        .with_user_preferences("Use British English.".into());

    let prompt = builder.build();

    assert!(prompt.contains("# Identity"));
    assert!(prompt.contains("# Tools"));
    assert!(prompt.contains("## read"));
    assert!(prompt.contains("# Skills"));
    assert!(prompt.contains("git-skill"));
    assert!(prompt.contains("# Context"));
    assert!(prompt.contains("AGENTS.md"));
    assert!(prompt.contains("# User Preferences"));
    assert!(prompt.contains("British English"));
}

#[test]
fn test_build_with_missing_components() {
    let builder = SystemPromptBuilder::new();
    let prompt = builder.build();

    assert!(prompt.contains("# Identity"));
    assert!(prompt.contains("No tools available"));
    assert!(prompt.contains("No skills available"));
    assert!(prompt.contains("No context files loaded"));
    // User preferences should not appear when empty
    assert!(!prompt.contains("# User Preferences"));
    // Append prompt should not appear when not set
    assert!(!prompt.contains("# Additional Instructions"));
}

// --- Custom prompt tests ---

#[test]
fn test_custom_prompt_replaces_identity() {
    let builder =
        SystemPromptBuilder::new().with_custom_prompt("You are a custom assistant.".into());

    let prompt = builder.build();

    assert!(prompt.contains("You are a custom assistant."));
    assert!(!prompt.contains("You are Talos"));
}

#[test]
fn test_custom_prompt_with_other_components() {
    let builder = SystemPromptBuilder::new()
        .with_custom_prompt("Custom identity.".into())
        .with_tools(vec![ToolDescription {
            name: "bash".into(),
            description: "Run commands".into(),
            ..Default::default()
        }]);

    let prompt = builder.build();

    assert!(prompt.contains("Custom identity."));
    assert!(prompt.contains("## bash"));
}

// --- Append prompt tests ---

#[test]
fn test_append_prompt_added_at_end() {
    let builder = SystemPromptBuilder::new().with_append_prompt("Always be concise.".into());

    let prompt = builder.build();

    assert!(prompt.contains("# Additional Instructions"));
    assert!(prompt.contains("Always be concise."));
    // Append should be the last section
    let append_pos = prompt
        .find("# Additional Instructions")
        .expect("append not found");
    let remaining = &prompt[append_pos..];
    assert!(!remaining[1..].contains("# Identity"));
    assert!(!remaining[1..].contains("# Tools"));
}

// --- Cache marker tests ---

#[test]
fn test_cache_marker_generation() {
    let builder = SystemPromptBuilder::new()
        .with_tools(vec![ToolDescription {
            name: "read".into(),
            description: "Read a file".into(),
            ..Default::default()
        }])
        .with_skill_index(vec![SkillIndex {
            name: "test-skill".into(),
            description: "A test skill".into(),
            triggers: vec!["test".into()],
            estimated_tokens: 0,
        }]);

    let (prompt, markers) = builder.build_with_cache_markers();

    assert_eq!(markers.len(), 3);

    // All markers should be Ephemeral type
    for marker in &markers {
        assert!(matches!(marker.cache_type, CacheType::Ephemeral));
    }

    // Markers should be in increasing order
    assert!(markers[0].offset < markers[1].offset);
    assert!(markers[1].offset < markers[2].offset);

    // Verify marker content matches prompt sections
    let identity_section = &prompt[markers[0].offset..markers[0].offset + markers[0].length];
    assert!(identity_section.contains("# Identity"));

    let tools_section = &prompt[markers[1].offset..markers[1].offset + markers[1].length];
    assert!(tools_section.contains("# Tools"));
    assert!(tools_section.contains("## read"));

    assert!(prompt.contains("# Skills"));
    assert!(prompt.contains("test-skill"));
}

#[test]
fn test_identity_template_slots_are_rendered() {
    let builder = SystemPromptBuilder::new()
        .with_template_var("custom_slot", "custom value")
        .with_workspace_info("Workspace root: /repo")
        .with_model_info("Model: test-model")
        .with_custom_prompt(
            "{{workspace_info}}\n{{model_info}}\n{{tool_protocol_hint}}\n{{custom_slot}}".into(),
        );

    let prompt = builder.build();

    assert!(prompt.contains("Workspace root: /repo"));
    assert!(prompt.contains("Model: test-model"));
    assert!(prompt.contains("Native tool calling is enabled"));
    assert!(prompt.contains("custom value"));
    assert!(!prompt.contains("{{workspace_info}}"));
    assert!(!prompt.contains("{{model_info}}"));
    assert!(!prompt.contains("{{tool_protocol_hint}}"));
}

#[test]
fn test_datetime_lives_after_cache_markers() {
    let builder = SystemPromptBuilder::new();
    let (prompt, markers) = builder.build_with_cache_markers();

    let runtime_pos = prompt
        .find("# Runtime Context")
        .expect("runtime context should be present");
    assert!(prompt.contains("Current datetime: unix_seconds="));
    for marker in markers {
        assert!(
            marker.offset + marker.length <= runtime_pos,
            "cache marker must not include dynamic datetime section"
        );
    }
}

#[test]
fn test_cache_markers_with_empty_components() {
    let builder = SystemPromptBuilder::new();
    let (_prompt, markers) = builder.build_with_cache_markers();

    assert_eq!(markers.len(), 3);

    // Identity marker
    assert!(markers[0].length > 0);
    // Tools marker (empty)
    assert!(markers[1].length > 0);
    // Skills marker (empty)
    assert!(markers[2].length > 0);
}

// --- Token estimation tests ---

#[test]
fn test_token_estimation() {
    let builder = SystemPromptBuilder::new();
    let tokens = builder.total_tokens();

    // Default identity is ~100 chars, so ~25 tokens minimum
    assert!(tokens > 10);
}

#[test]
fn test_token_estimation_with_content() {
    let builder = SystemPromptBuilder::new()
        .with_tools(vec![
            ToolDescription {
                name: "read".into(),
                description: "Read a file".into(),
                ..Default::default()
            },
            ToolDescription {
                name: "write".into(),
                description: "Write a file".into(),
                ..Default::default()
            },
        ])
        .with_context_files(vec![ContextFile {
            path: "AGENTS.md".into(),
            content: "A".repeat(1000),
        }]);

    let tokens = builder.total_tokens();
    // 1000 chars of context alone should be ~250 tokens
    assert!(tokens > 200);
}

// --- Prompt ordering tests ---

#[test]
fn test_prompt_ordering_identity_first() {
    let builder = SystemPromptBuilder::new().with_tools(vec![ToolDescription {
        name: "bash".into(),
        description: "Run commands".into(),
        ..Default::default()
    }]);

    let prompt = builder.build();

    let identity_pos = prompt.find("# Identity").expect("identity not found");
    let tools_pos = prompt.find("# Tools").expect("tools not found");

    assert!(
        identity_pos < tools_pos,
        "identity should come before tools"
    );
}

#[test]
fn test_prompt_ordering_tools_before_skills() {
    let builder = SystemPromptBuilder::new()
        .with_tools(vec![ToolDescription {
            name: "bash".into(),
            description: "Run commands".into(),
            ..Default::default()
        }])
        .with_skill_index(vec![SkillIndex {
            name: "test".into(),
            description: "Test skill".into(),
            triggers: vec![],
            estimated_tokens: 0,
        }]);

    let prompt = builder.build();

    let tools_pos = prompt.find("# Tools").expect("tools not found");
    let skills_pos = prompt.find("# Skills").expect("skills not found");

    assert!(tools_pos < skills_pos, "tools should come before skills");
}

#[test]
fn test_prompt_ordering_skills_before_context() {
    let builder = SystemPromptBuilder::new()
        .with_skill_index(vec![SkillIndex {
            name: "test".into(),
            description: "Test skill".into(),
            triggers: vec![],
            estimated_tokens: 0,
        }])
        .with_context_files(vec![ContextFile {
            path: "AGENTS.md".into(),
            content: "Rules".into(),
        }]);

    let prompt = builder.build();

    let skills_pos = prompt.find("# Skills").expect("skills not found");
    let context_pos = prompt.find("# Context").expect("context not found");

    assert!(
        skills_pos < context_pos,
        "skills should come before context"
    );
}

#[test]
fn test_prompt_ordering_context_before_preferences() {
    let builder = SystemPromptBuilder::new()
        .with_context_files(vec![ContextFile {
            path: "AGENTS.md".into(),
            content: "Rules".into(),
        }])
        .with_user_preferences("Be concise.".into());

    let prompt = builder.build();

    let context_pos = prompt.find("# Context").expect("context not found");
    let prefs_pos = prompt.find("# User Preferences").expect("prefs not found");

    assert!(
        context_pos < prefs_pos,
        "context should come before preferences"
    );
}

#[test]
fn test_prompt_ordering_append_last() {
    let builder = SystemPromptBuilder::new()
        .with_user_preferences("Be concise.".into())
        .with_append_prompt("Extra instructions.".into());

    let prompt = builder.build();

    let prefs_pos = prompt.find("# User Preferences").expect("prefs not found");
    let append_pos = prompt
        .find("# Additional Instructions")
        .expect("append not found");

    assert!(
        prefs_pos < append_pos,
        "preferences should come before append"
    );
}

// --- Tools sorting tests ---

#[test]
fn test_tools_sorted_alphabetically() {
    let builder = SystemPromptBuilder::new().with_tools(vec![
        ToolDescription {
            name: "write".into(),
            description: "Write a file".into(),
            ..Default::default()
        },
        ToolDescription {
            name: "bash".into(),
            description: "Run commands".into(),
            ..Default::default()
        },
        ToolDescription {
            name: "read".into(),
            description: "Read a file".into(),
            ..Default::default()
        },
    ]);

    let prompt = builder.build();

    let bash_pos = prompt.find("## bash").expect("bash not found");
    let read_pos = prompt.find("## read").expect("read not found");
    let write_pos = prompt.find("## write").expect("write not found");

    assert!(bash_pos < read_pos, "bash should come before read");
    assert!(read_pos < write_pos, "read should come before write");
}

// --- Log size test ---

#[test]
fn test_log_size_does_not_panic() {
    let builder = SystemPromptBuilder::new();
    // Should not panic, just prints to stderr
    builder.log_size();
}

// --- Default trait test ---

#[test]
fn test_default_builder() {
    let builder = SystemPromptBuilder::default();
    let prompt = builder.build();
    assert!(prompt.contains("# Identity"));
}

#[test]
fn activated_skill_context_is_cacheable_stable_prefix() {
    let builder = SystemPromptBuilder::new().with_activated_skill(Some(ActivatedSkillContext {
        name: "review".to_string(),
        content: "Review instructions stay provider-visible only.".to_string(),
    }));

    let stable = builder.build_stable_prefix();
    let dynamic = builder.build_dynamic_suffix();

    assert!(stable.contains("# Activated Skill: review"));
    assert!(stable.contains("Review instructions stay provider-visible only."));
    assert!(!dynamic.contains("Review instructions stay provider-visible only."));
}

// --- Clone test ---

#[test]
fn test_builder_clone() {
    let builder = SystemPromptBuilder::new().with_tools(vec![ToolDescription {
        name: "read".into(),
        description: "Read a file".into(),
        ..Default::default()
    }]);

    let cloned = builder.clone();
    let prompt1 = builder.build();
    let prompt2 = cloned.build();

    assert_eq!(prompt1, prompt2);
}

// --- ADR-015 required-asset tests ---

#[test]
fn test_required_prompt_assets_are_non_empty() {
    // ADR-015 requires tests proving required embedded prompt assets are non-empty.
    assert!(
        !DEFAULT_IDENTITY.trim().is_empty(),
        "identity.txt must be non-empty"
    );
    assert!(
        !TOOL_CALLING_FORMAT.trim().is_empty(),
        "tool_calling_format.txt must be non-empty"
    );
    assert!(
        !TOOL_CALLING_STRICT.trim().is_empty(),
        "tool_calling_strict.txt must be non-empty"
    );
    assert!(
        !MEMORY_PROMPT.trim().is_empty(),
        "memory.md must be non-empty"
    );
}

#[test]
fn test_identity_prompt_contains_talos_identity() {
    assert!(DEFAULT_IDENTITY.contains("Talos"));
}

// --- Memory section tests ---

#[test]
fn prompt_builder_with_memory_section() {
    let builder = SystemPromptBuilder::new().with_memory_section(Some("test memory".to_string()));

    let prompt = builder.build();

    assert!(prompt.contains("# Memory"), "Should contain Memory header");
    assert!(
        prompt.contains("test memory"),
        "Should contain memory content"
    );
}

#[test]
fn prompt_builder_without_memory_section() {
    let builder = SystemPromptBuilder::new();
    let prompt = builder.build();

    assert!(
        !prompt.contains("# Memory\n"),
        "Should not contain Memory header when no section set"
    );
}
