use super::*;
use talos_core::tool::ToolResourceKind;

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
    let engine = PermissionEngine::with_workspace_root(PathBuf::from("/tmp"));
    let decision = engine.evaluate(
        "find_references",
        &serde_json::json!({"name": "Tool", "file": "src/main.rs"}),
    );
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_workspace_auto_allow_path_param() {
    let engine = PermissionEngine::with_workspace_root(PathBuf::from("/tmp"));
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
    let mut engine2 = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
fn test_path_pattern_deny_outside_src() {
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
    assert_eq!(decision, PermissionDecision::Allow);
}

// --- Rule precedence tests ---

#[test]
fn test_first_match_wins() {
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
        decision: PermissionDecision::Deny("blocked".to_owned()),
        nature: None,
        resource: None,
        resource_kind: None,
    });

    let decision = engine.evaluate("bash", &serde_json::json!({"command": "ls"}));
    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_specific_rule_before_general() {
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
    assert_eq!(decision, PermissionDecision::Allow);

    let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
    assert_eq!(
        decision,
        PermissionDecision::Deny("write not allowed".to_owned())
    );
}

#[test]
fn test_runtime_allow_rule_bypasses_default_ask() {
    let mut engine = PermissionEngine::new();
    engine.add_runtime_allow_rule(PermissionRule::new_nature(
        ToolNature::Execute,
        Some("bash:read_only_inspection:abc".to_string()),
        Some(ResourceKind::Command),
        PermissionDecision::Allow,
    ));

    let profile = vec![ToolPermissionFacet::with_resource(
        ToolNature::Execute,
        "bash:read_only_inspection:abc",
        talos_core::tool::ToolResourceKind::Command,
    )];
    let decision = engine.evaluate_profile("bash", &profile, &serde_json::json!({}));

    assert_eq!(decision, PermissionDecision::Allow);
}

#[test]
fn test_runtime_allow_rule_does_not_override_deny() {
    let mut engine = PermissionEngine::new();
    engine
        .load_from_config(&serde_json::json!({
            "rules": [
                {
                    "decision": { "Deny": "shell blocked" },
                    "nature": "Execute",
                    "resource": "bash:*",
                    "resource_kind": "command"
                }
            ]
        }))
        .expect("deny rule config should load");
    engine.add_runtime_allow_rule(PermissionRule::new_nature(
        ToolNature::Execute,
        Some("bash:read_only_inspection:abc".to_string()),
        Some(ResourceKind::Command),
        PermissionDecision::Allow,
    ));

    let profile = vec![ToolPermissionFacet::with_resource(
        ToolNature::Execute,
        "bash:read_only_inspection:abc",
        talos_core::tool::ToolResourceKind::Command,
    )];
    let decision = engine.evaluate_profile("bash", &profile, &serde_json::json!({}));

    assert_eq!(
        decision,
        PermissionDecision::Deny("shell blocked".to_string())
    );
}

// --- Nature-based rule tests (T1) ---

#[test]
fn test_nature_match_without_resource_matches_all() {
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
        PermissionDecision::Deny("write not allowed".to_owned()),
    ));

    let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);

    let decision = engine.evaluate("edit", &serde_json::json!({"path": "src/lib.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);

    let decision = engine.evaluate("write", &serde_json::json!({"path": "Cargo.toml"}));
    assert_eq!(
        decision,
        PermissionDecision::Deny("write not allowed".to_owned())
    );
}

#[test]
fn test_nature_domain_resource_match() {
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
    engine.add_rule(PermissionRule::new(
        "write",
        Some("src/**".to_owned()),
        PermissionDecision::Allow,
    ));
    engine.add_rule(PermissionRule::new(
        "write",
        None,
        PermissionDecision::Deny("write not allowed".to_owned()),
    ));

    let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);

    let decision = engine.evaluate("write", &serde_json::json!({"path": "Cargo.toml"}));
    assert_eq!(
        decision,
        PermissionDecision::Deny("write not allowed".to_owned())
    );
}

#[test]
fn test_first_match_wins_nature_rules() {
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
        PermissionDecision::Deny("write not allowed".to_owned()),
    ));

    let decision = engine.evaluate("write", &serde_json::json!({"path": "src/main.rs"}));
    assert_eq!(decision, PermissionDecision::Allow);

    let decision = engine.evaluate("write", &serde_json::json!({"path": "Cargo.toml"}));
    assert_eq!(
        decision,
        PermissionDecision::Deny("write not allowed".to_owned())
    );
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
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
                "decision": "Deny"
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
    assert_eq!(decision, PermissionDecision::Deny("".to_owned()));
}

#[test]
fn test_default_ruleset_is_nature_form() {
    let engine = PermissionEngine::new();
    // Default ruleset should have exactly 5 rules (one per ToolNature variant)
    assert_eq!(engine.rules.len(), 5);
    for rule in &engine.rules {
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
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
    let mut engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
    let engine = PermissionEngine {
        rules: Vec::new(),
        workspace_root: None,
        trusted_workspace: false,
    };
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
