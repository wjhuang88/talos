use super::*;
use talos_core::tool::{ToolProvenance, ToolResourceKind};

// --- Default ruleset tests ---

#[test]
fn test_default_read_tool_allowed() {
    let engine = PermissionEngine::new();
    let decision = engine.evaluate("read", &serde_json::json!({"path": "Cargo.toml"}));
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_default_list_tool_allowed() {
    let engine = PermissionEngine::new();
    let decision = engine.evaluate("list", &serde_json::json!({"path": "src"}));
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_default_write_tool_ask() {
    let engine = PermissionEngine::new();
    let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
    assert_eq!(decision, PermissionDecision::Ask);
}

#[test]
fn test_default_edit_tool_ask() {
    let engine = PermissionEngine::new();
    let decision = engine.evaluate("edit", &serde_json::json!({"path": "src/main.rs"}));
    assert_eq!(decision, PermissionDecision::Ask);
}

#[test]
fn test_default_bash_tool_ask() {
    let engine = PermissionEngine::new();
    let decision = engine.evaluate("bash", &serde_json::json!({"command": "ls"}));
    assert_eq!(decision, PermissionDecision::Ask);
}

#[test]
fn test_default_grep_tool_allowed() {
    let engine = PermissionEngine::new();
    let decision = engine.evaluate("grep", &serde_json::json!({"pattern": "fn"}));
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_default_glob_tool_allowed() {
    let engine = PermissionEngine::new();
    let decision = engine.evaluate("glob", &serde_json::json!({"pattern": "*.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_default_ls_tool_allowed() {
    let engine = PermissionEngine::new();
    let decision = engine.evaluate("ls", &serde_json::json!({}));
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_default_delete_tool_ask() {
    let engine = PermissionEngine::new();
    let decision = engine.evaluate("delete", &serde_json::json!({"path": "temp.txt"}));
    assert_eq!(decision, PermissionDecision::Ask);
}

#[test]
fn test_default_find_symbol_allowed() {
    let engine = PermissionEngine::new();
    let decision = engine.evaluate("find_symbol", &serde_json::json!({"name": "Tool"}));
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_default_find_references_allowed() {
    let engine = PermissionEngine::new();
    let decision = engine.evaluate(
        "find_references",
        &serde_json::json!({"name": "Tool", "file": "src/main.rs"}),
    );
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_default_list_symbols_allowed() {
    let engine = PermissionEngine::new();
    let decision = engine.evaluate("list_symbols", &serde_json::json!({"path": "src/lib.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_default_list_imports_allowed() {
    let engine = PermissionEngine::new();
    let decision = engine.evaluate("list_imports", &serde_json::json!({"file": "src/main.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_workspace_auto_allow_file_param() {
    let engine = PermissionEngine::with_workspace_root(std::env::temp_dir());
    let decision = engine.evaluate(
        "find_references",
        &serde_json::json!({"name": "Tool", "file": "src/main.rs"}),
    );
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_workspace_auto_allow_path_param() {
    let engine = PermissionEngine::with_workspace_root(std::env::temp_dir());
    let decision = engine.evaluate(
        "find_symbol",
        &serde_json::json!({"name": "Tool", "path": "src/main.rs"}),
    );
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_default_unknown_tool_ask() {
    let engine = PermissionEngine::new();
    let decision = engine.evaluate("custom_tool", &serde_json::json!({}));
    assert_eq!(decision, PermissionDecision::Ask);
}

// --- Custom rule tests ---

#[test]
fn test_custom_rule_allow_bash() {
    let mut engine = PermissionEngine::new();
    engine.add_rule(PermissionRule {
        tool_name: "bash".to_owned(),
        path_pattern: None,
        decision: PermissionDecision::Allow,
        nature: None,
        resource: None,
        resource_kind: None,
    });

    // Custom rule is appended, so default bash rule still matches first
    // We need to test with a new engine where we control rule order
    let mut engine2 = PermissionEngine::empty();
    engine2.add_rule(PermissionRule {
        tool_name: "bash".to_owned(),
        path_pattern: None,
        decision: PermissionDecision::Allow,
        nature: None,
        resource: None,
        resource_kind: None,
    });

    let decision = engine2.evaluate("bash", &serde_json::json!({"command": "ls"}));
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_custom_rule_deny_write_to_sensitive_path() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule {
        tool_name: "write".to_owned(),
        path_pattern: Some(".env".to_owned()),
        decision: PermissionDecision::Deny("sensitive file".to_owned()),
        nature: None,
        resource: None,
        resource_kind: None,
    });

    let decision = engine.evaluate("write", &serde_json::json!({"path": ".env"}));
    assert_eq!(
        decision,
        PermissionDecision::Deny("sensitive file".to_owned())
    );
}

// --- Path pattern matching tests ---

#[test]
fn test_path_pattern_src_glob_matches() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule {
        tool_name: "read".to_owned(),
        path_pattern: Some("src/**/*.rs".to_owned()),
        decision: PermissionDecision::Allow,
        nature: None,
        resource: None,
        resource_kind: None,
    });

    let decision = engine.evaluate("read", &serde_json::json!({"path": "src/main.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_path_pattern_src_glob_nested() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule {
        tool_name: "read".to_owned(),
        path_pattern: Some("src/**/*.rs".to_owned()),
        decision: PermissionDecision::Allow,
        nature: None,
        resource: None,
        resource_kind: None,
    });

    let decision = engine.evaluate("read", &serde_json::json!({"path": "src/utils/helpers.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_path_pattern_src_glob_no_match() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule {
        tool_name: "read".to_owned(),
        path_pattern: Some("src/**/*.rs".to_owned()),
        decision: PermissionDecision::Allow,
        nature: None,
        resource: None,
        resource_kind: None,
    });

    let decision = engine.evaluate("read", &serde_json::json!({"path": "tests/main.rs"}));
    // No rule matches, default for "read" is Allow
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_general_deny_dominates_specific_path_allow() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule {
        tool_name: "write".to_owned(),
        path_pattern: Some("src/**/*.rs".to_owned()),
        decision: PermissionDecision::Allow,
        nature: None,
        resource: None,
        resource_kind: None,
    });
    engine.add_rule(PermissionRule {
        tool_name: "write".to_owned(),
        path_pattern: None,
        decision: PermissionDecision::Deny("only src allowed".to_owned()),
        nature: None,
        resource: None,
        resource_kind: None,
    });

    let decision = engine.evaluate("write", &serde_json::json!({"path": "tests/main.rs"}));
    assert_eq!(
        decision,
        PermissionDecision::Deny("only src allowed".to_owned())
    );

    let decision = engine.evaluate("write", &serde_json::json!({"path": "src/lib.rs"}));
    assert_eq!(
        decision,
        PermissionDecision::Deny("only src allowed".to_owned())
    );
}

// --- Rule precedence tests ---

#[test]
fn test_first_non_deny_match_wins() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule {
        tool_name: "bash".to_owned(),
        path_pattern: None,
        decision: PermissionDecision::Allow,
        nature: None,
        resource: None,
        resource_kind: None,
    });
    engine.add_rule(PermissionRule {
        tool_name: "bash".to_owned(),
        path_pattern: None,
        decision: PermissionDecision::Ask,
        nature: None,
        resource: None,
        resource_kind: None,
    });

    let decision = engine.evaluate("bash", &serde_json::json!({"command": "ls"}));
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_specific_rule_before_general() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule {
        tool_name: "write".to_owned(),
        path_pattern: Some("tmp/**".to_owned()),
        decision: PermissionDecision::Allow,
        nature: None,
        resource: None,
        resource_kind: None,
    });
    engine.add_rule(PermissionRule {
        tool_name: "write".to_owned(),
        path_pattern: None,
        decision: PermissionDecision::Deny("write not allowed".to_owned()),
        nature: None,
        resource: None,
        resource_kind: None,
    });

    let decision = engine.evaluate("write", &serde_json::json!({"path": "tmp/out.txt"}));
    assert_eq!(
        decision,
        PermissionDecision::Deny("write not allowed".to_owned()),
        "every matching policy Deny must dominate an earlier Allow"
    );

    let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
    assert_eq!(
        decision,
        PermissionDecision::Deny("write not allowed".to_owned())
    );
}

#[test]
fn test_session_grant_resolves_default_ask() {
    let state = PermissionSessionState::new(PermissionEngine::new());
    let profile = vec![ToolPermissionFacet::with_resource(
        ToolNature::Execute,
        "bash:read_only_inspection:abc",
        talos_core::tool::ToolResourceKind::Command,
    )];
    let input = serde_json::json!({"command": "git status"});
    let request = PermissionRequest::native("bash", &profile, &input);
    let context = PermissionContext::compatibility();
    let proposal = state
        .propose(&request, &context, GrantScope::Session)
        .expect("proposal");
    let pending = state
        .approve_session(proposal, &request, &context, GrantSource::InteractiveHuman)
        .expect("session approval");
    state.admit(pending, &request, &context).expect("admission");

    assert_eq!(
        state
            .evaluate(&request, &context)
            .expect("evaluation")
            .decision(),
        PermissionDecision::Allow
    );
}

#[test]
fn test_session_grant_does_not_override_later_deny() {
    let mut engine = PermissionEngine::new();
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Execute,
        Some("bash:read_only_inspection:abc".to_string()),
        Some(ResourceKind::Command),
        PermissionDecision::Ask,
    ));
    let profile = vec![ToolPermissionFacet::with_resource(
        ToolNature::Execute,
        "bash:read_only_inspection:abc",
        talos_core::tool::ToolResourceKind::Command,
    )];
    let input = serde_json::json!({"command": "git status"});
    let request = PermissionRequest::native("bash", &profile, &input);
    let context = PermissionContext::compatibility();
    let state = PermissionSessionState::new(engine);
    let proposal = state
        .propose(&request, &context, GrantScope::Session)
        .expect("proposal");
    state
        .approve_session(proposal, &request, &context, GrantSource::InteractiveHuman)
        .expect("session approval");
    state
        .replace_policy(PermissionEngine::from_rules(vec![
            PermissionRule::new_nature(
                ToolNature::Execute,
                Some("bash:read_only_inspection:abc".to_string()),
                Some(ResourceKind::Command),
                PermissionDecision::Deny("shell blocked".to_string()),
            ),
        ]))
        .expect("policy replacement");

    assert_eq!(
        state
            .evaluate(&request, &context)
            .expect("evaluation")
            .decision(),
        PermissionDecision::Deny("shell blocked".to_string()),
        "a later matching policy Deny must shadow an installed Session grant"
    );
}

// --- Nature-based rule tests (T1) ---

#[test]
fn test_nature_match_without_resource_matches_all() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Write,
        None,
        None,
        PermissionDecision::Allow,
    ));

    let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);

    let decision = engine.evaluate("edit", &serde_json::json!({"path": "src/main.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);

    let decision = engine.evaluate("delete", &serde_json::json!({"path": "tmp.txt"}));
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_nature_path_resource_match() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Write,
        Some("src/**".to_owned()),
        Some(ResourceKind::Path),
        PermissionDecision::Allow,
    ));
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Write,
        None,
        None,
        PermissionDecision::Ask,
    ));

    let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);

    let decision = engine.evaluate("edit", &serde_json::json!({"path": "src/lib.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);

    let decision = engine.evaluate("write", &serde_json::json!({"path": "Cargo.toml"}));
    assert_eq!(decision, PermissionDecision::Ask);
}

#[test]
fn test_nature_domain_resource_match() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Network,
        Some("api.github.com".to_owned()),
        Some(ResourceKind::Domain),
        PermissionDecision::Allow,
    ));
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Network,
        None,
        None,
        PermissionDecision::Ask,
    ));

    let decision = engine.evaluate(
        "http_request",
        &serde_json::json!({"url": "https://api.github.com/repos"}),
    );
    assert_eq!(decision, PermissionDecision::Allow);

    let decision = engine.evaluate(
        "http_request",
        &serde_json::json!({"url": "https://example.com/api"}),
    );
    assert_eq!(decision, PermissionDecision::Ask);
}

#[test]
fn test_profile_denies_when_any_facet_is_denied() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Network,
        Some("example.com".to_owned()),
        Some(ResourceKind::Domain),
        PermissionDecision::Allow,
    ));
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Write,
        Some("blocked/**".to_owned()),
        Some(ResourceKind::Path),
        PermissionDecision::Deny("write blocked".to_owned()),
    ));

    let profile = vec![
        ToolPermissionFacet::with_resource(
            ToolNature::Network,
            "example.com",
            ToolResourceKind::Domain,
        ),
        ToolPermissionFacet::with_resource(
            ToolNature::Write,
            "blocked/file.txt",
            ToolResourceKind::Path,
        ),
    ];

    let decision = engine.evaluate_profile("save_url", &profile, &serde_json::json!({}));
    assert_eq!(
        decision,
        PermissionDecision::Deny("write blocked".to_owned())
    );
}

#[test]
fn test_profile_asks_when_any_facet_requires_approval() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Network,
        Some("example.com".to_owned()),
        Some(ResourceKind::Domain),
        PermissionDecision::Allow,
    ));
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Write,
        None,
        None,
        PermissionDecision::Ask,
    ));

    let profile = vec![
        ToolPermissionFacet::with_resource(
            ToolNature::Network,
            "example.com",
            ToolResourceKind::Domain,
        ),
        ToolPermissionFacet::with_resource(
            ToolNature::Write,
            "out/file.txt",
            ToolResourceKind::Path,
        ),
    ];

    let decision = engine.evaluate_profile("save_url", &profile, &serde_json::json!({}));
    assert_eq!(decision, PermissionDecision::Ask);
}

#[test]
fn test_extractor_write_from_destination() {
    let input = serde_json::json!({"destination": "downloads/file.txt"});
    let result = ResourceExtractor::extract(ToolNature::Write, &input);
    assert_eq!(result, Some("downloads/file.txt".to_owned()));
}

#[test]
fn test_legacy_tool_name_rule_still_works() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule::new(
        "write",
        Some("src/**".to_owned()),
        PermissionDecision::Allow,
    ));
    engine.add_rule(PermissionRule::new("write", None, PermissionDecision::Ask));

    let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);

    let decision = engine.evaluate("write", &serde_json::json!({"path": "Cargo.toml"}));
    assert_eq!(decision, PermissionDecision::Ask);
}

#[test]
fn test_first_non_deny_match_wins_for_nature_rules() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Write,
        Some("src/**".to_owned()),
        Some(ResourceKind::Path),
        PermissionDecision::Allow,
    ));
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Write,
        None,
        None,
        PermissionDecision::Ask,
    ));

    let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);

    let decision = engine.evaluate("write", &serde_json::json!({"path": "Cargo.toml"}));
    assert_eq!(decision, PermissionDecision::Ask);
}

// --- ResourceExtractor tests (T2) ---

#[test]
fn test_extractor_read_from_path() {
    let input = serde_json::json!({"path": "src/main.rs"});
    let result = ResourceExtractor::extract(ToolNature::Read, &input);
    assert_eq!(result, Some("src/main.rs".to_owned()));
}

#[test]
fn test_extractor_read_from_file_fallback() {
    let input = serde_json::json!({"name": "Tool", "file": "src/lib.rs"});
    let result = ResourceExtractor::extract(ToolNature::Read, &input);
    assert_eq!(result, Some("src/lib.rs".to_owned()));
}

#[test]
fn test_extractor_write_from_path() {
    let input = serde_json::json!({"path": "src/main.rs", "content": "hello"});
    let result = ResourceExtractor::extract(ToolNature::Write, &input);
    assert_eq!(result, Some("src/main.rs".to_owned()));
}

#[test]
fn test_extractor_execute_first_token() {
    let input = serde_json::json!({"command": "scripts/deploy.sh --arg1 --arg2"});
    let result = ResourceExtractor::extract(ToolNature::Execute, &input);
    assert_eq!(result, Some("scripts/deploy.sh".to_owned()));
}

#[test]
fn test_extractor_execute_single_word() {
    let input = serde_json::json!({"command": "cargo"});
    let result = ResourceExtractor::extract(ToolNature::Execute, &input);
    assert_eq!(result, Some("cargo".to_owned()));
}

#[test]
fn test_extractor_network_host_extraction() {
    let input = serde_json::json!({"url": "https://api.github.com/repos"});
    let result = ResourceExtractor::extract(ToolNature::Network, &input);
    assert_eq!(result, Some("api.github.com".to_owned()));
}

#[test]
fn test_extractor_network_host_lowercase() {
    let input = serde_json::json!({"url": "https://API.GITHUB.COM/repos"});
    let result = ResourceExtractor::extract(ToolNature::Network, &input);
    assert_eq!(result, Some("api.github.com".to_owned()));
}

#[test]
fn test_extractor_network_host_no_port() {
    let input = serde_json::json!({"url": "https://api.github.com:443/repos"});
    let result = ResourceExtractor::extract(ToolNature::Network, &input);
    assert_eq!(result, Some("api.github.com".to_owned()));
}

#[test]
fn test_extractor_network_invalid_url() {
    let input = serde_json::json!({"url": "not-a-url"});
    let result = ResourceExtractor::extract(ToolNature::Network, &input);
    assert_eq!(result, None);
}

#[test]
fn test_extractor_missing_field_returns_none() {
    let input = serde_json::json!({});
    assert_eq!(ResourceExtractor::extract(ToolNature::Read, &input), None);
    assert_eq!(ResourceExtractor::extract(ToolNature::Write, &input), None);
    assert_eq!(
        ResourceExtractor::extract(ToolNature::Execute, &input),
        None
    );
    assert_eq!(
        ResourceExtractor::extract(ToolNature::Network, &input),
        None
    );
}

// --- Load from config tests ---

#[test]
fn test_load_from_config() {
    let mut engine = PermissionEngine::new();
    let config = serde_json::json!({
        "rules": [
            {
                "tool_name": "bash",
                "path_pattern": null,
                "decision": "Allow"
            },
            {
                "tool_name": "write",
                "path_pattern": "src/**/*.rs",
                "decision": "Deny"
            }
        ]
    });

    engine
        .load_from_config(&config)
        .expect("config should load");

    // Custom bash rule is prepended, so it matches first
    let decision = engine.evaluate("bash", &serde_json::json!({"command": "ls"}));
    assert_eq!(decision, PermissionDecision::Allow);

    // Write to src/ is denied by custom rule
    let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
    assert_eq!(decision, PermissionDecision::Deny("".to_owned()));
}

#[test]
fn test_load_from_config_invalid() {
    let mut engine = PermissionEngine::new();
    let config = serde_json::json!({"not_rules": []});

    let result = engine.load_from_config(&config);
    assert!(result.is_err());
}

#[test]
fn test_load_from_config_malformed_rule() {
    let mut engine = PermissionEngine::new();
    let config = serde_json::json!({
        "rules": [
            {"tool_name": 123}
        ]
    });

    let result = engine.load_from_config(&config);
    assert!(result.is_err());
}

#[test]
fn test_load_old_config_format_tool_name_only() {
    let mut engine = PermissionEngine::empty();
    let config = serde_json::json!({
        "rules": [
            {
                "tool_name": "write",
                "path_pattern": "src/**",
                "decision": "Allow"
            },
            {
                "tool_name": "write",
                "decision": "Ask"
            }
        ]
    });

    engine
        .load_from_config(&config)
        .expect("config should load");

    // Old format: tool_name-based matching with inferred nature
    let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);

    let decision = engine.evaluate("write", &serde_json::json!({"path": "Cargo.toml"}));
    assert_eq!(decision, PermissionDecision::Ask);
}

#[test]
fn test_load_new_config_format_nature_form() {
    let mut engine = PermissionEngine::empty();
    let config = serde_json::json!({
        "rules": [
            {
                "nature": "Write",
                "resource": "src/**",
                "resource_kind": "path",
                "decision": "Allow"
            },
            {
                "nature": "Write",
                "decision": "Ask"
            }
        ]
    });

    engine
        .load_from_config(&config)
        .expect("config should load");

    // New format: nature-based matching
    let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);

    let decision = engine.evaluate("edit", &serde_json::json!({"path": "src/lib.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);

    let decision = engine.evaluate("write", &serde_json::json!({"path": "Cargo.toml"}));
    assert_eq!(decision, PermissionDecision::Ask);
}

#[test]
fn test_default_ruleset_is_nature_form() {
    let engine = PermissionEngine::new();
    // Default ruleset should have exactly 5 rules (one per ToolNature variant)
    assert_eq!(engine.rules().len(), 5);
    for rule in engine.rules() {
        assert!(
            rule.nature.is_some(),
            "default rules should use nature form"
        );
    }
}

#[test]
fn test_default_internal_tool_allowed() {
    let engine = PermissionEngine::new();
    let decision =
        engine.evaluate_with_nature("todo_create", ToolNature::Internal, &serde_json::json!({}));
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_config_with_both_tool_name_and_nature_prefers_nature() {
    let mut engine = PermissionEngine::empty();
    // Rule has both tool_name AND nature set — nature should take precedence
    let config = serde_json::json!({
        "rules": [
            {
                "tool_name": "read",
                "nature": "Write",
                "resource": "src/**",
                "resource_kind": "path",
                "decision": "Allow"
            }
        ]
    });

    engine
        .load_from_config(&config)
        .expect("config should load");

    // Nature is Write, so it matches write tools, not read tools
    let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);

    // Read tool doesn't match the Write nature rule
    let decision = engine.evaluate("read", &serde_json::json!({"path": "src/main.rs"}));
    assert_eq!(decision, PermissionDecision::Allow); // falls through to default Read Allow
}

// --- Edge cases ---

#[test]
fn test_tool_name_case_insensitive_default() {
    let engine = PermissionEngine::new();
    // Default decision uses lowercase comparison
    let decision = engine.evaluate("READ", &serde_json::json!({}));
    assert_eq!(decision, PermissionDecision::Allow);

    let decision = engine.evaluate("Write", &serde_json::json!({}));
    assert_eq!(decision, PermissionDecision::Ask);
}

#[test]
fn test_tool_name_exact_match_in_rules() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule {
        tool_name: "read".to_owned(),
        path_pattern: None,
        decision: PermissionDecision::Allow,
        nature: None,
        resource: None,
        resource_kind: None,
    });

    // Rule matching is case-sensitive
    let decision = engine.evaluate("READ", &serde_json::json!({}));
    // No rule matches, falls through to default which is case-insensitive
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_empty_rules_falls_to_default() {
    let engine = PermissionEngine::empty();
    let decision = engine.evaluate("read", &serde_json::json!({}));
    assert_eq!(decision, PermissionDecision::Allow);

    let decision = engine.evaluate("write", &serde_json::json!({}));
    assert_eq!(decision, PermissionDecision::Ask);
}

#[test]
fn test_permission_rule_new() {
    let rule = PermissionRule::new(
        "bash",
        Some("safe/**".to_owned()),
        PermissionDecision::Allow,
    );
    assert_eq!(rule.tool_name, "bash");
    assert_eq!(rule.path_pattern, Some("safe/**".to_owned()));
    assert_eq!(rule.decision, PermissionDecision::Allow);
}

#[test]
fn test_permission_decision_serialization() {
    let allow = PermissionDecision::Allow;
    let json = serde_json::to_string(&allow).expect("serialize");
    assert_eq!(json, "\"Allow\"");

    let deny = PermissionDecision::Deny("nope".to_owned());
    let json = serde_json::to_string(&deny).expect("serialize");
    assert_eq!(json, "{\"Deny\":\"nope\"}");

    let ask = PermissionDecision::Ask;
    let json = serde_json::to_string(&ask).expect("serialize");
    assert_eq!(json, "\"Ask\"");
}

#[test]
fn test_permission_rule_serialization() {
    let rule = PermissionRule {
        tool_name: "write".to_owned(),
        path_pattern: Some("src/**".to_owned()),
        decision: PermissionDecision::Ask,
        nature: None,
        resource: None,
        resource_kind: None,
    };
    let json = serde_json::to_string(&rule).expect("serialize");
    let parsed: PermissionRule = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.tool_name, "write");
    assert_eq!(parsed.path_pattern, Some("src/**".to_owned()));
    assert_eq!(parsed.decision, PermissionDecision::Ask);
}

#[test]
fn trusted_workspace_allows_repo_write_but_preserves_deny() {
    let root = std::env::temp_dir().join(format!("talos-trust-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("workspace");
    let mut engine = PermissionEngine::with_workspace_root(root.clone());
    engine.set_trusted_workspace(true);

    assert_eq!(
        engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"})),
        PermissionDecision::Allow
    );

    engine
        .load_from_config(&serde_json::json!({"rules": [{
            "nature": "Write",
            "resource": "secrets/**",
            "resource_kind": "path",
            "decision": {"Deny": "sensitive path"}
        }]}))
        .expect("load deny rule");
    assert_eq!(
        engine.evaluate("write", &serde_json::json!({"path": "secrets/key"})),
        PermissionDecision::Deny("sensitive path".to_string())
    );
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn workspace_path_rejects_relative_escape() {
    let root = std::env::temp_dir().join(format!("talos-path-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("workspace");
    assert!(!is_path_in_workspace("../../outside", &root));
    std::fs::remove_dir_all(root).ok();
}

// --- ADR-040 Evidence-based command enforcement tests ---

#[test]
fn evidence_unknown_command_escalates_under_trust() {
    let root = std::env::temp_dir().join(format!("talos-ev-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("workspace");
    let mut engine = PermissionEngine::with_workspace_root(root.clone());
    engine.set_trusted_workspace(true);

    let evidence = crate::access_evidence::AccessEvidence::unknown();
    let decision = engine.evaluate_command_with_evidence(
        "bash",
        "some-command --flag",
        &evidence,
        &serde_json::json!({"command": "some-command --flag"}),
    );
    assert_eq!(decision, PermissionDecision::Ask);
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn evidence_network_command_escalates_under_trust() {
    let root = std::env::temp_dir().join(format!("talos-ev-net-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("workspace");
    let mut engine = PermissionEngine::with_workspace_root(root.clone());
    engine.set_trusted_workspace(true);

    let evidence = crate::access_evidence::AccessEvidence::network();
    let decision = engine.evaluate_command_with_evidence(
        "bash",
        "curl https://example.com",
        &evidence,
        &serde_json::json!({"command": "curl https://example.com"}),
    );
    assert_eq!(decision, PermissionDecision::Ask);
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn evidence_deny_rule_overrides_trust() {
    let root = std::env::temp_dir().join(format!("talos-ev-deny-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("workspace");
    let mut engine = PermissionEngine::with_workspace_root(root.clone());
    engine.set_trusted_workspace(true);
    engine
        .load_from_config(&serde_json::json!({"rules": [{
            "nature": "Execute",
            "decision": {"Deny": "all execute blocked by policy"}
        }]}))
        .expect("load deny rule");

    let evidence = crate::access_evidence::AccessEvidence::declared_read(vec![]);
    let decision = engine.evaluate_command_with_evidence(
        "bash",
        "cat Cargo.toml",
        &evidence,
        &serde_json::json!({"command": "cat Cargo.toml"}),
    );
    assert_eq!(
        decision,
        PermissionDecision::Deny("all execute blocked by policy".to_string())
    );
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn evidence_pipe_command_is_unknown_and_escalates() {
    let root = std::env::temp_dir().join(format!("talos-ev-pipe-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("workspace");
    let mut engine = PermissionEngine::with_workspace_root(root.clone());
    engine.set_trusted_workspace(true);

    let evidence = crate::access_evidence::classify_command_access("cat foo | grep bar");
    assert!(evidence.is_unknown());

    let decision = engine.evaluate_command_with_evidence(
        "bash",
        "cat foo | grep bar",
        &evidence,
        &serde_json::json!({"command": "cat foo | grep bar"}),
    );
    assert_eq!(decision, PermissionDecision::Ask);
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn evidence_traversal_command_escalates() {
    let root = std::env::temp_dir().join(format!("talos-ev-trav-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("workspace");
    let mut engine = PermissionEngine::with_workspace_root(root.clone());
    engine.set_trusted_workspace(true);

    let evidence = crate::access_evidence::AccessEvidence {
        kind: crate::access_evidence::AccessKind::Read,
        state: crate::access_evidence::EvidenceState::Declared,
        paths: vec![std::path::PathBuf::from("/etc/passwd")],
        detail: String::new(),
    };

    let decision = engine.evaluate_command_with_evidence(
        "bash",
        "cat /etc/passwd",
        &evidence,
        &serde_json::json!({"command": "cat /etc/passwd"}),
    );
    assert_eq!(decision, PermissionDecision::Ask);
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn evidence_spawn_command_escalates_under_trust() {
    let root = std::env::temp_dir().join(format!("talos-ev-spawn-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("workspace");
    let mut engine = PermissionEngine::with_workspace_root(root.clone());
    engine.set_trusted_workspace(true);

    let evidence = crate::access_evidence::AccessEvidence::spawn();
    let decision = engine.evaluate_command_with_evidence(
        "bash",
        "sh -c 'something'",
        &evidence,
        &serde_json::json!({"command": "sh -c 'something'"}),
    );
    assert_eq!(decision, PermissionDecision::Ask);
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn trust_revoke_clears_persisted_trust() {
    let root = std::env::temp_dir().join(format!("talos-revoke-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("workspace");
    let workspace = std::env::temp_dir().join(format!("talos-revoke-ws-{}", std::process::id()));
    std::fs::create_dir_all(&workspace).expect("ws");

    let store = crate::WorkspaceTrustStore::new(&root);
    assert!(!store.is_trusted(&workspace));

    store.grant_trust(&workspace).expect("grant");
    assert!(store.is_trusted(&workspace));

    store.revoke_trust(&workspace).expect("revoke");
    assert!(!store.is_trusted(&workspace));

    let store2 = crate::WorkspaceTrustStore::new(&root);
    assert!(
        !store2.is_trusted(&workspace),
        "revocation must persist across instances"
    );

    std::fs::remove_dir_all(root).ok();
    std::fs::remove_dir_all(workspace).ok();
}

#[test]
fn non_git_workspace_commands_always_ask() {
    let root = std::env::temp_dir().join(format!("talos-nongit-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("workspace");
    let mut engine = PermissionEngine::with_workspace_root(root.clone());
    engine.set_trusted_workspace(true);

    let evidence = crate::access_evidence::AccessEvidence::declared_read(vec![root.clone()]);
    let decision = engine.evaluate_command_with_evidence(
        "bash",
        "cat file.txt",
        &evidence,
        &serde_json::json!({"command": "cat file.txt"}),
    );
    assert_eq!(
        decision,
        PermissionDecision::Ask,
        "non-Git workspace should not get command trust even if evidence is clean"
    );
    std::fs::remove_dir_all(root).ok();
}

#[test]
fn evidence_never_produces_allow_by_itself() {
    let root = std::env::temp_dir().join(format!("talos-noallow-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("workspace");
    let mut engine = PermissionEngine::with_workspace_root(root.clone());
    engine.set_trusted_workspace(true);

    let evidence = crate::access_evidence::AccessEvidence {
        kind: crate::access_evidence::AccessKind::Read,
        state: crate::access_evidence::EvidenceState::Declared,
        paths: vec![root.join("Cargo.toml")],
        detail: "cat".to_string(),
    };

    std::fs::write(root.join("Cargo.toml"), "[package]\n").ok();

    let decision = engine.evaluate_command_with_evidence(
        "bash",
        "cat Cargo.toml",
        &evidence,
        &serde_json::json!({"command": "cat Cargo.toml"}),
    );
    assert_eq!(
        decision,
        PermissionDecision::Ask,
        "evidence must NEVER produce Allow by itself — it is observation, not authority"
    );
    std::fs::remove_dir_all(root).ok();
}

// ── SEC-001: external path authorization tests ─────────────────────────────

#[test]
fn external_read_path_requires_ask_not_allow() {
    use talos_core::tool::{ToolNature, ToolPermissionFacet};

    let root =
        std::env::temp_dir().join(format!("talos-sec001-ext-{}/workspace", std::process::id()));
    let external = std::env::temp_dir().join(format!("talos-sec001-out-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("workspace");
    std::fs::write(&external, "secret").expect("external file");

    let engine = PermissionEngine::with_workspace_root(root.clone());

    let facet = ToolPermissionFacet::new(ToolNature::Read);
    let input = serde_json::json!({"path": external.to_string_lossy().to_string()});
    let decision = engine.evaluate_facet("read", &facet, &input);

    assert_eq!(
        decision,
        PermissionDecision::Ask,
        "external read must require Ask, not Allow (SEC-001)"
    );

    std::fs::remove_dir_all(&root).ok();
    std::fs::remove_file(&external).ok();
}

#[test]
fn internal_read_path_still_allowed() {
    use talos_core::tool::{ToolNature, ToolPermissionFacet};

    let root = std::env::temp_dir().join(format!("talos-sec001-int-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("workspace");
    std::fs::write(root.join("inside.txt"), "data").expect("file");

    let engine = PermissionEngine::with_workspace_root(root.clone());

    let facet = ToolPermissionFacet::new(ToolNature::Read);
    let input = serde_json::json!({"path": "inside.txt"});
    let decision = engine.evaluate_facet("read", &facet, &input);

    assert_eq!(
        decision,
        PermissionDecision::Allow,
        "internal read must remain Allow (SEC-001 preserves workspace behavior)"
    );

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn deny_rule_still_wins_for_external_path() {
    use talos_core::tool::{ToolNature, ToolPermissionFacet};

    let root = std::env::temp_dir().join(format!("talos-sec001-deny-{}", std::process::id()));
    let external =
        std::env::temp_dir().join(format!("talos-sec001-deny-out-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("workspace");
    std::fs::write(&external, "data").expect("external file");

    let mut engine = PermissionEngine::with_workspace_root(root);
    let config = serde_json::json!({"rules": [{"nature": "Read", "decision": "Deny"}]});
    engine
        .load_from_config(&config)
        .expect("operation should succeed");

    let facet = ToolPermissionFacet::new(ToolNature::Read);
    let input = serde_json::json!({"path": external.to_string_lossy().to_string()});
    let decision = engine.evaluate_facet("read", &facet, &input);

    assert!(
        matches!(decision, PermissionDecision::Deny(_)),
        "Deny rule must win over external-path Ask (SEC-001)"
    );

    std::fs::remove_file(&external).ok();
}

#[test]
fn exact_session_grant_is_reused_for_external_path() {
    use talos_core::tool::{ToolNature, ToolPermissionFacet, ToolResourceKind};

    let root = std::env::temp_dir().join(format!("talos-sec001-allow-{}", std::process::id()));
    let external =
        std::env::temp_dir().join(format!("talos-sec001-allow-out-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("workspace");
    std::fs::write(&external, "data").expect("external file");

    let state = PermissionSessionState::new(PermissionEngine::with_workspace_root(root));
    let facet = ToolPermissionFacet::with_resource(
        ToolNature::Read,
        external.to_string_lossy(),
        ToolResourceKind::Path,
    );
    let input = serde_json::json!({"path": external.to_string_lossy()});

    let facets = [facet];
    let request = PermissionRequest::native("read", &facets, &input);
    let context = PermissionContext::compatibility();
    let proposal = state
        .propose(&request, &context, GrantScope::Session)
        .expect("external path proposal");
    state
        .approve_session(proposal, &request, &context, GrantSource::InteractiveHuman)
        .expect("external path approval");
    assert_eq!(
        state
            .evaluate(&request, &context)
            .expect("evaluation")
            .decision(),
        PermissionDecision::Allow
    );

    std::fs::remove_file(&external).ok();
}

#[test]
fn read_facet_without_path_keeps_default_allow() {
    use talos_core::tool::{ToolNature, ToolPermissionFacet};

    let root = std::env::temp_dir().join(format!("talos-sec001-no-path-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("workspace");
    let engine = PermissionEngine::with_workspace_root(root.clone());

    assert_eq!(
        engine.evaluate_facet(
            "remote_metadata",
            &ToolPermissionFacet::new(ToolNature::Read),
            &serde_json::json!({})
        ),
        PermissionDecision::Allow,
        "a read-only tool with no path resource is not an external file access"
    );

    std::fs::remove_dir_all(root).ok();
}

// --- I189 structured decision report tests ---

fn report_rule_source(report: &PermissionDecisionReport) -> PermissionRuleSource {
    match report.facets()[0].source() {
        PermissionDecisionSource::Rule { rule_source, .. } => *rule_source,
        other => panic!("expected rule source, got {other:?}"),
    }
}

#[test]
fn structured_report_distinguishes_policy_rule_sources() {
    let input = serde_json::json!({"path": "src/lib.rs"});
    let facets = [ToolPermissionFacet::with_resource(
        ToolNature::Read,
        "src/lib.rs",
        ToolResourceKind::Path,
    )];
    let context = PermissionContext::compatibility();

    let default_engine = PermissionEngine::new();
    let report = default_engine.evaluate_request(
        &PermissionRequest::native("read", &facets, &input),
        &context,
    );
    assert_eq!(report_rule_source(&report), PermissionRuleSource::Default);
    assert_eq!(report.facets()[0].reason(), PermissionReason::RuleAllow);

    let mut configured_engine = PermissionEngine::empty();
    configured_engine
        .load_from_config(&serde_json::json!({"rules": [{
            "nature": "Read",
            "decision": "Allow"
        }]}))
        .expect("configured rule");
    let report = configured_engine.evaluate_request(
        &PermissionRequest::native("read", &facets, &input),
        &context,
    );
    assert_eq!(
        report_rule_source(&report),
        PermissionRuleSource::Configured
    );

    let mut explicit_engine = PermissionEngine::empty();
    explicit_engine.add_rule(PermissionRule::new_nature(
        ToolNature::Read,
        None,
        None,
        PermissionDecision::Allow,
    ));
    let report = explicit_engine.evaluate_request(
        &PermissionRequest::native("read", &facets, &input),
        &context,
    );
    assert_eq!(report_rule_source(&report), PermissionRuleSource::Explicit);
}

#[test]
fn structured_report_identifies_workspace_sources() {
    let root = std::env::temp_dir().join(format!("talos-i189-ws-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("workspace");
    let input = serde_json::json!({"path": "inside.txt"});
    let facets = [ToolPermissionFacet::with_resource(
        ToolNature::Write,
        "inside.txt",
        ToolResourceKind::Path,
    )];

    let mut trusted = PermissionEngine::with_workspace_root(root.clone());
    trusted.set_trusted_workspace(true);
    let report = trusted.evaluate_request(
        &PermissionRequest::native("write", &facets, &input),
        &PermissionContext::compatibility(),
    );
    assert_eq!(report.outcome(), PermissionOutcome::Allow);
    assert_eq!(
        report.facets()[0].reason(),
        PermissionReason::TrustedWorkspaceWrite
    );
    assert_eq!(
        report.facets()[0].source(),
        &PermissionDecisionSource::WorkspaceTrust
    );

    let external = std::env::temp_dir().join(format!("talos-i189-external-{}", std::process::id()));
    let external_input = serde_json::json!({"path": external.to_string_lossy()});
    let external_facets = [ToolPermissionFacet::with_resource(
        ToolNature::Read,
        external.to_string_lossy(),
        ToolResourceKind::Path,
    )];
    let bounded = PermissionEngine::with_workspace_root(root.clone());
    let report = bounded.evaluate_request(
        &PermissionRequest::native("read", &external_facets, &external_input),
        &PermissionContext::compatibility(),
    );
    assert_eq!(report.outcome(), PermissionOutcome::Ask);
    assert_eq!(
        report.facets()[0].reason(),
        PermissionReason::ExternalPathRequiresApproval
    );
    assert_eq!(
        report.facets()[0].source(),
        &PermissionDecisionSource::WorkspaceBoundary
    );

    std::fs::remove_dir_all(root).ok();
}

#[test]
fn compatibility_entrypoints_equal_structured_projection() {
    let engine = PermissionEngine::new();
    let cases = [
        ("read", ToolNature::Read, serde_json::json!({"path": "a"})),
        ("write", ToolNature::Write, serde_json::json!({"path": "b"})),
        (
            "bash",
            ToolNature::Execute,
            serde_json::json!({"command": "true"}),
        ),
        (
            "http_request",
            ToolNature::Network,
            serde_json::json!({"url": "https://example.com"}),
        ),
        ("todo_create", ToolNature::Internal, serde_json::json!({})),
    ];

    for (tool_name, nature, input) in cases {
        let facets = [ToolPermissionFacet::new(nature)];
        let structured = engine.evaluate_request(
            &PermissionRequest::native(tool_name, &facets, &input),
            &PermissionContext::compatibility(),
        );
        assert_eq!(
            engine.evaluate_with_nature(tool_name, nature, &input),
            structured.decision()
        );
        assert_eq!(
            engine.evaluate_profile(tool_name, &facets, &input),
            structured.decision()
        );
        assert_eq!(engine.evaluate(tool_name, &input), structured.decision());
    }
}

#[test]
fn structured_aggregate_severity_is_order_independent_and_complete() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Read,
        None,
        None,
        PermissionDecision::Allow,
    ));
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Execute,
        None,
        None,
        PermissionDecision::Ask,
    ));
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Write,
        None,
        None,
        PermissionDecision::Deny("write blocked".to_string()),
    ));
    let read = ToolPermissionFacet::new(ToolNature::Read);
    let execute = ToolPermissionFacet::new(ToolNature::Execute);
    let write = ToolPermissionFacet::new(ToolNature::Write);
    let permutations = [
        [read.clone(), execute.clone(), write.clone()],
        [read.clone(), write.clone(), execute.clone()],
        [execute.clone(), read.clone(), write.clone()],
        [execute.clone(), write.clone(), read.clone()],
        [write.clone(), read.clone(), execute.clone()],
        [write, execute, read],
    ];
    let input = serde_json::json!({});

    for facets in permutations {
        let report = engine.evaluate_request(
            &PermissionRequest::native("hybrid", &facets, &input),
            &PermissionContext::compatibility(),
        );
        assert_eq!(report.outcome(), PermissionOutcome::Deny);
        assert_eq!(
            report.decision(),
            PermissionDecision::Deny("write blocked".to_string())
        );
        assert_eq!(report.facets().len(), 3, "all facets must be reported");
    }
}

#[test]
fn resource_less_nature_allow_does_not_cover_background_command() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Execute,
        None,
        None,
        PermissionDecision::Allow,
    ));
    let facets = [ToolPermissionFacet::with_resource(
        ToolNature::Execute,
        "background:bash:bash:validation_build:template:abc:cargo:check",
        ToolResourceKind::Command,
    )];
    let input = serde_json::json!({"command": "cargo check", "background": true});

    let report = engine.evaluate_request(
        &PermissionRequest::native("bash", &facets, &input),
        &PermissionContext::compatibility(),
    );

    assert_eq!(report.outcome(), PermissionOutcome::Ask);
}

#[test]
fn resource_less_legacy_allow_does_not_cover_background_command() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule::new("bash", None, PermissionDecision::Allow));
    let facets = [ToolPermissionFacet::with_resource(
        ToolNature::Execute,
        "background:bash:bash:complex_shell:exact:abc",
        ToolResourceKind::Command,
    )];
    let input = serde_json::json!({"command": "sleep 1", "background": true});

    let report = engine.evaluate_request(
        &PermissionRequest::native("bash", &facets, &input),
        &PermissionContext::compatibility(),
    );

    assert_eq!(report.outcome(), PermissionOutcome::Ask);
}

#[test]
fn background_command_preserves_deny_and_explicit_allow_rules() {
    let resource = "background:exec:exec:pwd:/usr/bin/sleep";
    let facet = ToolPermissionFacet::with_resource(
        ToolNature::Execute,
        resource,
        ToolResourceKind::Command,
    );
    let input = serde_json::json!({"command": "sleep", "args": ["1"], "background": true});

    let mut denied = PermissionEngine::empty();
    denied.add_rule(PermissionRule::new_nature(
        ToolNature::Execute,
        None,
        None,
        PermissionDecision::Deny("execute disabled".to_owned()),
    ));
    assert_eq!(
        denied
            .evaluate_request(
                &PermissionRequest::native("exec", std::slice::from_ref(&facet), &input),
                &PermissionContext::compatibility(),
            )
            .decision(),
        PermissionDecision::Deny("execute disabled".to_owned())
    );

    let mut allowed = PermissionEngine::empty();
    allowed.add_rule(PermissionRule::new_nature(
        ToolNature::Execute,
        Some(resource.to_owned()),
        Some(crate::resource::ResourceKind::Command),
        PermissionDecision::Allow,
    ));
    assert_eq!(
        allowed
            .evaluate_request(
                &PermissionRequest::native("exec", &[facet], &input),
                &PermissionContext::compatibility(),
            )
            .outcome(),
        PermissionOutcome::Allow
    );
}

#[test]
fn compatibility_projection_preserves_first_deny_message() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Read,
        None,
        None,
        PermissionDecision::Deny("read blocked".to_string()),
    ));
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Write,
        None,
        None,
        PermissionDecision::Deny("write blocked".to_string()),
    ));
    let input = serde_json::json!({});
    let read_first = [
        ToolPermissionFacet::new(ToolNature::Read),
        ToolPermissionFacet::new(ToolNature::Write),
    ];
    let write_first = [
        ToolPermissionFacet::new(ToolNature::Write),
        ToolPermissionFacet::new(ToolNature::Read),
    ];

    assert_eq!(
        engine.evaluate_profile("hybrid", &read_first, &input),
        PermissionDecision::Deny("read blocked".to_string())
    );
    assert_eq!(
        engine.evaluate_profile("hybrid", &write_first, &input),
        PermissionDecision::Deny("write blocked".to_string())
    );
}

#[test]
fn observer_report_and_debug_exclude_private_values() {
    let sentinels = [
        "secret-tool-name",
        "secret-server-name",
        "secret-plugin-name",
        "secret-plugin-version",
        "secret-plugin-carrier",
        "secret-resource-path",
        "secret-description",
        "secret-input-token",
        "secret-deny-reason",
    ];
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Write,
        None,
        None,
        PermissionDecision::Deny("secret-deny-reason".to_string()),
    ));
    let facets = [ToolPermissionFacet::with_resource(
        ToolNature::Write,
        "secret-resource-path",
        ToolResourceKind::Path,
    )
    .with_description("secret-description")];
    let input = serde_json::json!({
        "path": "secret-resource-path",
        "api_key": "secret-input-token"
    });
    let provenances = [
        ToolProvenance::McpRemote {
            server: "secret-server-name".to_string(),
        },
        ToolProvenance::Plugin {
            name: "secret-plugin-name".to_string(),
            version: "secret-plugin-version".to_string(),
            carrier: "secret-plugin-carrier".to_string(),
        },
    ];

    for provenance in provenances {
        let request = PermissionRequest::new("secret-tool-name", provenance, &facets, &input);
        let report = engine.evaluate_request(&request, &PermissionContext::compatibility());
        let surfaces = [
            serde_json::to_string(&report).expect("serialize safe report"),
            format!("{report:?}"),
            format!("{request:?}"),
        ];

        for surface in surfaces {
            for sentinel in sentinels {
                assert!(!surface.contains(sentinel), "leaked {sentinel}: {surface}");
            }
        }
        assert_eq!(
            report.decision(),
            PermissionDecision::Deny("secret-deny-reason".to_string()),
            "compatibility projection must retain the original denial"
        );
    }
}

#[test]
fn missing_consequential_resource_stays_ask_and_is_reported() {
    let engine = PermissionEngine::empty();
    let input = serde_json::json!({"url": "not a URL"});
    let facets = [ToolPermissionFacet::new(ToolNature::Network)];
    let report = engine.evaluate_request(
        &PermissionRequest::native("http_request", &facets, &input),
        &PermissionContext::compatibility(),
    );

    assert_eq!(report.decision(), PermissionDecision::Ask);
    assert_eq!(
        report.facets()[0].resource_state(),
        PermissionResourceState::MissingOrInvalid
    );
    assert_eq!(
        report.facets()[0].reason(),
        PermissionReason::MissingOrInvalidResource
    );
    assert_eq!(
        report.facets()[0].reason().message(),
        "consequential facet has no usable concrete resource"
    );
    assert_eq!(
        report.facets()[0].source(),
        &PermissionDecisionSource::DefaultBehavior
    );
}

#[test]
fn empty_profile_inference_and_modes_do_not_change_policy() {
    let engine = PermissionEngine::new();
    let input = serde_json::json!({"path": "out.txt"});
    let modes = [
        PermissionMode::ReadOnly,
        PermissionMode::Interactive,
        PermissionMode::Headless,
        PermissionMode::TrustedWorkspace,
        PermissionMode::Auto,
    ];

    for mode in modes {
        let report = engine.evaluate_request(
            &PermissionRequest::native("write", &[], &input),
            &PermissionContext::new(mode, InteractionCapability::Unavailable),
        );
        assert_eq!(report.mode(), mode);
        assert_eq!(report.interaction(), InteractionCapability::Unavailable);
        assert_eq!(report.decision(), PermissionDecision::Ask);
        assert_eq!(report.facets().len(), 1);
        assert_eq!(report.facets()[0].nature(), ToolNature::Write);
    }
}

#[test]
fn rule_ids_survive_unrelated_insertions() {
    let mut engine = PermissionEngine::empty();
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Read,
        None,
        None,
        PermissionDecision::Allow,
    ));
    let input = serde_json::json!({});
    let facets = [ToolPermissionFacet::new(ToolNature::Read)];
    let evaluate_id = |engine: &PermissionEngine| {
        let report = engine.evaluate_request(
            &PermissionRequest::native("read", &facets, &input),
            &PermissionContext::compatibility(),
        );
        match report.facets()[0].source() {
            PermissionDecisionSource::Rule { rule_id, .. } => *rule_id,
            other => panic!("expected rule, got {other:?}"),
        }
    };
    let before = evaluate_id(&engine);
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Write,
        None,
        None,
        PermissionDecision::Ask,
    ));
    assert_eq!(evaluate_id(&engine), before);
    engine
        .load_from_config(&serde_json::json!({"rules": [{
            "nature": "Network",
            "decision": "Ask"
        }]}))
        .expect("prepend unrelated configured rule");
    assert_eq!(evaluate_id(&engine), before);
}
