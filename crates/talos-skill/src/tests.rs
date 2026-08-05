use super::*;
use crate::loader::home_dir;
use crate::parser::{split_frontmatter, validate_frontmatter};
use std::fs;
use std::path::{Path, PathBuf};

// -----------------------------------------------------------------------
// Test fixtures
// -----------------------------------------------------------------------

fn valid_skill_content() -> String {
    r#"---
name: test-skill
description: A test skill for unit testing
triggers:
  - test
  - unit
---

# Test Skill

This is the body of the test skill.

## Instructions

1. Do something
2. Do something else
"#
    .to_string()
}

fn no_frontmatter_content() -> String {
    "# No Frontmatter\n\nThis file has no YAML frontmatter.".to_string()
}

fn invalid_yaml_content() -> String {
    r#"---
name: [invalid yaml
description: test
---

Body content.
"#
    .to_string()
}

fn missing_fields_content() -> String {
    r#"---
name: incomplete-skill
---

Body content.
"#
    .to_string()
}

// -----------------------------------------------------------------------
// Parsing tests
// -----------------------------------------------------------------------

#[test]
fn parse_valid_skill_md() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let path = dir.path().join("SKILL.md");
    fs::write(&path, valid_skill_content()).expect("failed to write test file");

    let skill = SkillLoader::parse(&path).expect("parsing should succeed");

    assert_eq!(skill.name, "test-skill");
    assert_eq!(skill.description, "A test skill for unit testing");
    assert_eq!(skill.triggers, vec!["test", "unit"]);
    assert!(skill.body.contains("# Test Skill"));
    assert_eq!(skill.source_path, path);
}

#[test]
fn parse_skill_without_frontmatter_errors() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let path = dir.path().join("SKILL.md");
    fs::write(&path, no_frontmatter_content()).expect("failed to write test file");

    let result = SkillLoader::parse(&path);
    assert!(result.is_err());

    match result.expect_err("operation should fail") {
        SkillError::InvalidFrontmatter(msg) => {
            assert!(msg.contains("---"));
        }
        other => panic!("expected InvalidFrontmatter, got: {other:?}"),
    }
}

#[test]
fn parse_invalid_yaml_errors() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let path = dir.path().join("SKILL.md");
    fs::write(&path, invalid_yaml_content()).expect("failed to write test file");

    let result = SkillLoader::parse(&path);
    assert!(result.is_err());

    match result.expect_err("operation should fail") {
        SkillError::YamlParseError(_) => {}
        other => panic!("expected YamlParseError, got: {other:?}"),
    }
}

#[test]
fn parse_missing_required_fields_errors() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let path = dir.path().join("SKILL.md");
    fs::write(&path, missing_fields_content()).expect("failed to write test file");

    let result = SkillLoader::parse(&path);
    assert!(result.is_err());

    // serde_yaml reports missing fields as YamlParseError
    match result.expect_err("operation should fail") {
        SkillError::YamlParseError(e) => {
            assert!(e.to_string().contains("description"));
        }
        other => panic!("expected YamlParseError, got: {other:?}"),
    }
}

#[test]
fn parse_nonexistent_file_errors() {
    let path = Path::new("/nonexistent/path/SKILL.md");
    let result = SkillLoader::parse(path);
    assert!(result.is_err());

    match result.expect_err("operation should fail") {
        SkillError::FileNotFound(p) => {
            assert_eq!(p, path);
        }
        other => panic!("expected FileNotFound, got: {other:?}"),
    }
}

// -----------------------------------------------------------------------
// Discovery tests
// -----------------------------------------------------------------------

#[test]
fn discover_from_single_directory() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let skills_dir = dir.path().join("skills");
    fs::create_dir(&skills_dir).expect("failed to create skills dir");

    fs::write(skills_dir.join("SKILL.md"), valid_skill_content()).expect("failed to write skill");

    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let skills = loader.discover().expect("discovery should succeed");
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].name, "test-skill");
}

#[test]
fn discover_from_multiple_directories() {
    let dir1 = tempfile::tempdir().expect("failed to create temp dir");
    let dir2 = tempfile::tempdir().expect("failed to create temp dir");

    let skills1 = dir1.path().join("skills");
    let skills2 = dir2.path().join("skills");
    fs::create_dir(&skills1).expect("failed to create skills dir");
    fs::create_dir(&skills2).expect("failed to create skills dir");

    let skill_a = r#"---
name: skill-a
description: First skill
triggers:
  - alpha
---

Body A.
"#;

    let skill_b = r#"---
name: skill-b
description: Second skill
triggers:
  - beta
---

Body B.
"#;

    fs::write(skills1.join("SKILL.md"), skill_a).expect("failed to write skill A");
    fs::write(skills2.join("SKILL.md"), skill_b).expect("failed to write skill B");

    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills1, skills2],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let skills = loader.discover().expect("discovery should succeed");
    assert_eq!(skills.len(), 2);

    let names: Vec<&str> = skills.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"skill-a"));
    assert!(names.contains(&"skill-b"));
}

#[test]
fn discover_deduplicates_by_name() {
    let dir1 = tempfile::tempdir().expect("failed to create temp dir");
    let dir2 = tempfile::tempdir().expect("failed to create temp dir");

    let skills1 = dir1.path().join("skills");
    let skills2 = dir2.path().join("skills");
    fs::create_dir(&skills1).expect("failed to create skills dir");
    fs::create_dir(&skills2).expect("failed to create skills dir");

    // Same name, different descriptions
    let skill_v1 = r#"---
name: duplicate-skill
description: Version 1
triggers:
  - dup
---

Body V1.
"#;

    let skill_v2 = r#"---
name: duplicate-skill
description: Version 2
triggers:
  - dup
---

Body V2.
"#;

    fs::write(skills1.join("SKILL.md"), skill_v1).expect("failed to write skill V1");
    fs::write(skills2.join("SKILL.md"), skill_v2).expect("failed to write skill V2");

    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills1, skills2],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let skills = loader.discover().expect("discovery should succeed");
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].description, "Version 1"); // first wins
}

#[test]
fn discover_skips_non_skill_files() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let skills_dir = dir.path().join("skills");
    fs::create_dir(&skills_dir).expect("failed to create skills dir");

    // Write a valid SKILL.md
    fs::write(skills_dir.join("SKILL.md"), valid_skill_content()).expect("failed to write skill");

    // Write a non-skill file
    fs::write(skills_dir.join("README.md"), "# Not a skill").expect("failed to write readme");

    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let skills = loader.discover().expect("discovery should succeed");
    assert_eq!(skills.len(), 1);
}

#[test]
fn discover_skips_unparseable_files() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let skills_dir = dir.path().join("skills");
    fs::create_dir(&skills_dir).expect("failed to create skills dir");

    fs::write(skills_dir.join("SKILL.md"), valid_skill_content()).expect("failed to write skill");

    fs::create_dir_all(skills_dir.join("nested")).expect("failed to create nested dir");
    fs::write(skills_dir.join("nested/SKILL.md"), "no frontmatter")
        .expect("failed to write invalid skill");

    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let skills = loader.discover().expect("discovery should succeed");
    assert_eq!(skills.len(), 1);
}

// -----------------------------------------------------------------------
// Index tests
// -----------------------------------------------------------------------

#[test]
fn skill_index_generation() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let skills_dir = dir.path().join("skills");
    fs::create_dir(&skills_dir).expect("failed to create skills dir");

    let skill_a = r#"---
name: skill-a
description: Description A
triggers:
  - trigger-a
---

Body A.
"#;

    let skill_b = r#"---
name: skill-b
description: Description B
triggers:
  - trigger-b
  - trigger-b2
---

Body B.
"#;

    fs::create_dir(skills_dir.join("skill-a")).expect("failed to create skill-a dir");
    fs::create_dir(skills_dir.join("skill-b")).expect("failed to create skill-b dir");
    fs::write(skills_dir.join("skill-a/SKILL.md"), skill_a).expect("failed to write skill A");
    fs::write(skills_dir.join("skill-b/SKILL.md"), skill_b).expect("failed to write skill B");

    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    loader.discover().expect("discovery should succeed");
    let index = loader.get_index();

    assert_eq!(index.len(), 2);

    let names: Vec<&str> = index.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"skill-a"));
    assert!(names.contains(&"skill-b"));

    // Verify index entries don't contain body content
    for entry in &index {
        assert!(!entry.description.contains("Body"));
    }
}

#[test]
fn skill_index_empty_when_no_skills() {
    let loader = SkillLoader {
        skills: Vec::new(),
        search_paths: Vec::new(),
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let index = loader.get_index();
    assert!(index.is_empty());
}

// -----------------------------------------------------------------------
// Trigger matching tests
// -----------------------------------------------------------------------

#[test]
fn trigger_matching_exact() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let skills_dir = dir.path().join("skills");
    fs::create_dir(&skills_dir).expect("failed to create skills dir");

    let skill = r#"---
name: git-skill
description: Git operations
triggers:
  - git
  - commit
  - push
---

Git instructions.
"#;

    fs::write(skills_dir.join("SKILL.md"), skill).expect("failed to write skill");

    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    loader.discover().expect("discovery should succeed");

    // Check that triggers are correctly parsed
    let skill = &loader.skills[0];
    assert!(skill.triggers.contains(&"git".to_string()));
    assert!(skill.triggers.contains(&"commit".to_string()));
    assert!(skill.triggers.contains(&"push".to_string()));
}

#[test]
fn trigger_matching_case_sensitive() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let skills_dir = dir.path().join("skills");
    fs::create_dir(&skills_dir).expect("failed to create skills dir");

    let skill = r#"---
name: case-skill
description: Case sensitivity test
triggers:
  - Git
  - GIT
---

Body.
"#;

    fs::write(skills_dir.join("SKILL.md"), skill).expect("failed to write skill");

    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    loader.discover().expect("discovery should succeed");

    let skill = &loader.skills[0];
    assert!(skill.triggers.contains(&"Git".to_string()));
    assert!(skill.triggers.contains(&"GIT".to_string()));
    // Triggers preserve case as specified
}

#[test]
fn trigger_matching_empty_triggers() {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let skills_dir = dir.path().join("skills");
    fs::create_dir(&skills_dir).expect("failed to create skills dir");

    let skill = r#"---
name: no-triggers
description: A skill with no triggers
triggers: []
---

Body.
"#;

    fs::write(skills_dir.join("SKILL.md"), skill).expect("failed to write skill");

    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    loader.discover().expect("discovery should succeed");

    let skill = &loader.skills[0];
    assert!(skill.triggers.is_empty());
}

// -----------------------------------------------------------------------
// Frontmatter splitting tests
// -----------------------------------------------------------------------

#[test]
fn split_frontmatter_basic() {
    let content = "---\nname: test\n---\n\nBody content.";
    let (fm, body) = split_frontmatter(content).expect("split should succeed");
    assert_eq!(fm, "name: test");
    assert_eq!(body.trim(), "Body content.");
}

#[test]
fn split_frontmatter_with_leading_whitespace() {
    let content = "  \n---\nname: test\n---\n\nBody.";
    let (fm, _body) = split_frontmatter(content).expect("split should succeed");
    assert_eq!(fm, "name: test");
}

#[test]
fn split_frontmatter_multiline_yaml() {
    let content = "---\nname: test\ndescription: A skill\ntriggers:\n  - a\n  - b\n---\n\nBody.";
    let (fm, body) = split_frontmatter(content).expect("split should succeed");
    assert!(fm.contains("name: test"));
    assert!(fm.contains("description: A skill"));
    assert!(fm.contains("- a"));
    assert_eq!(body.trim(), "Body.");
}

#[test]
fn split_frontmatter_no_opening_delimiter() {
    let content = "name: test\n---\n\nBody.";
    let result = split_frontmatter(content);
    assert!(result.is_err());
}

#[test]
fn split_frontmatter_no_closing_delimiter() {
    let content = "---\nname: test\n\nBody.";
    let result = split_frontmatter(content);
    assert!(result.is_err());
}

// -----------------------------------------------------------------------
// Validation tests
// -----------------------------------------------------------------------

#[test]
fn validate_empty_name() {
    let fm = SkillFrontmatter {
        name: String::new(),
        description: "A skill".into(),
        triggers: vec![],
    };
    let result = validate_frontmatter(&fm);
    assert!(result.is_err());
}

#[test]
fn validate_empty_description() {
    let fm = SkillFrontmatter {
        name: "test".into(),
        description: String::new(),
        triggers: vec![],
    };
    let result = validate_frontmatter(&fm);
    assert!(result.is_err());
}

#[test]
fn validate_valid_frontmatter() {
    let fm = SkillFrontmatter {
        name: "test".into(),
        description: "A skill".into(),
        triggers: vec!["trigger".into()],
    };
    assert!(validate_frontmatter(&fm).is_ok());
}

// -----------------------------------------------------------------------
// Default implementation test
// -----------------------------------------------------------------------

#[test]
fn skill_loader_default() {
    let loader = SkillLoader::default();
    assert!(loader.skills.is_empty());
    // search_paths may or may not be empty depending on filesystem state
}

// -----------------------------------------------------------------------
// Token estimation tests
// -----------------------------------------------------------------------

#[test]
fn estimate_tokens_english_text() {
    // "hello world" = 11 chars → ~3 tokens (11/4 ≈ 2.75, rounded up)
    let tokens = estimate_tokens("hello world");
    assert!(tokens >= 2 && tokens <= 4);
}

#[test]
fn estimate_tokens_empty_string() {
    assert_eq!(estimate_tokens(""), 0);
}

#[test]
fn estimate_tokens_cjk_text() {
    // CJK: 2 chars per token
    let cjk = "你好世界"; // 4 chars → ~2 tokens
    let tokens = estimate_tokens(cjk);
    assert!(tokens >= 1 && tokens <= 3);
}

#[test]
fn estimate_tokens_mixed_text() {
    // Mixed English and CJK
    let mixed = "Hello 世界";
    let tokens = estimate_tokens(mixed);
    assert!(tokens >= 2);
}

#[test]
fn estimate_tokens_long_text_scales() {
    let text = "a".repeat(400);
    let tokens = estimate_tokens(&text);
    // 400 chars / 4 ≈ 100 tokens
    assert!(tokens >= 90 && tokens <= 110);
}

// -----------------------------------------------------------------------
// SkillManager tests
// -----------------------------------------------------------------------

fn make_skill(name: &str, description: &str, triggers: &[&str], body: &str) -> Skill {
    Skill {
        name: name.to_string(),
        description: description.to_string(),
        triggers: triggers.iter().map(|s| s.to_string()).collect(),
        body: body.to_string(),
        source_path: PathBuf::from(format!("/tmp/skills/{name}/SKILL.md")),
        source: SkillSource::default(),
    }
}

#[test]
fn skill_manager_level0_index_generation() {
    let loader = SkillLoader {
        skills: vec![
            make_skill("git", "Git operations", &["git", "commit"], "body"),
            make_skill("test", "Unit testing", &["test", "unit"], "body"),
        ],
        search_paths: Vec::new(),
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let mut manager = SkillManager::new(loader);
    let index = manager.get_index();

    assert_eq!(index.len(), 2);
    assert_eq!(index[0].name, "git");
    assert_eq!(index[1].name, "test");
    // Each entry should have estimated_tokens > 0
    assert!(index[0].estimated_tokens > 0);
    assert!(index[1].estimated_tokens > 0);
}

#[test]
fn skill_manager_index_cached() {
    let loader = SkillLoader {
        skills: vec![make_skill("a", "desc a", &["a"], "body")],
        search_paths: Vec::new(),
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let mut manager = SkillManager::new(loader);
    let index1 = manager.get_index();
    let ptr1 = index1.as_ptr();
    let _ = index1;
    let index2 = manager.get_index();
    let ptr2 = index2.as_ptr();

    // Same pointer — cached
    assert!(std::ptr::eq(ptr1, ptr2));
}

#[test]
fn skill_manager_get_index_tokens() {
    let loader = SkillLoader {
        skills: vec![
            make_skill("a", "short", &["a"], "body"),
            make_skill("b", "short", &["b"], "body"),
        ],
        search_paths: Vec::new(),
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let mut manager = SkillManager::new(loader);
    let total = manager.get_index_tokens();
    assert!(total > 0);
}

#[test]
fn skill_manager_get_index_tokens_under_3000_for_20_skills() {
    let skills: Vec<Skill> = (0..20)
        .map(|i| {
            make_skill(
                &format!("skill-{i}"),
                &format!("Description for skill number {i}"),
                &[&format!("trigger-{i}")],
                "body",
            )
        })
        .collect();

    let loader = SkillLoader {
        skills,
        search_paths: Vec::new(),
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let mut manager = SkillManager::new(loader);
    let total = manager.get_index_tokens();
    assert!(
        total < 3000,
        "Level 0 index should be under 3000 tokens for 20 skills, got {total}"
    );
}

#[test]
fn skill_manager_load_skill_level1() {
    let skill = make_skill(
        "my-skill",
        "Does something",
        &["do"],
        "# Full body\n\nInstructions here.",
    );
    let loader = SkillLoader {
        skills: vec![skill],
        search_paths: Vec::new(),
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let mut manager = SkillManager::new(loader);
    let loaded = manager.load_skill("my-skill").expect("should load");

    assert_eq!(loaded.name, "my-skill");
    assert!(loaded.body.contains("# Full body"));
    assert!(loaded.body.contains("Instructions here"));
}

#[test]
fn skill_manager_load_skill_not_found() {
    let loader = SkillLoader {
        skills: vec![make_skill("exists", "desc", &["x"], "body")],
        search_paths: Vec::new(),
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let mut manager = SkillManager::new(loader);
    let result = manager.load_skill("nonexistent");
    assert!(result.is_err());
}

#[test]
fn skill_manager_load_skill_idempotent() {
    let skill = make_skill("idempotent", "desc", &["x"], "body");
    let loader = SkillLoader {
        skills: vec![skill],
        search_paths: Vec::new(),
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let mut manager = SkillManager::new(loader);
    manager.load_skill("idempotent").expect("first load");
    manager.load_skill("idempotent").expect("second load");

    // Only one entry in active_skills
    assert_eq!(manager.get_active_skills().len(), 1);
}

#[test]
fn skill_manager_unload_skill() {
    let skill = make_skill("unload-me", "desc", &["x"], "body");
    let loader = SkillLoader {
        skills: vec![skill],
        search_paths: Vec::new(),
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let mut manager = SkillManager::new(loader);
    manager.load_skill("unload-me").expect("should load");
    assert_eq!(manager.get_active_skills().len(), 1);

    manager.unload_skill("unload-me");
    assert_eq!(manager.get_active_skills().len(), 0);

    // Can reload
    manager.load_skill("unload-me").expect("should reload");
    assert_eq!(manager.get_active_skills().len(), 1);
}

#[test]
fn skill_manager_get_active_skills() {
    let skills = vec![
        make_skill("a", "desc a", &["a"], "body a"),
        make_skill("b", "desc b", &["b"], "body b"),
    ];
    let loader = SkillLoader {
        skills,
        search_paths: Vec::new(),
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let mut manager = SkillManager::new(loader);
    assert!(manager.get_active_skills().is_empty());

    manager.load_skill("a").expect("load a");
    assert_eq!(manager.get_active_skills().len(), 1);

    manager.load_skill("b").expect("load b");
    assert_eq!(manager.get_active_skills().len(), 2);
}

#[test]
fn skill_manager_match_skill_exact() {
    let skills = vec![
        make_skill("git", "Git operations", &["git", "commit", "push"], "body"),
        make_skill("test", "Testing", &["test", "unit"], "body"),
    ];
    let loader = SkillLoader {
        skills,
        search_paths: Vec::new(),
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let manager = SkillManager::new(loader);

    assert_eq!(
        manager.match_skill("I need to git push"),
        Some("git".to_string())
    );
    assert_eq!(
        manager.match_skill("run unit tests"),
        Some("test".to_string())
    );
}

#[test]
fn skill_manager_match_skill_case_insensitive() {
    let skills = vec![make_skill(
        "git",
        "Git operations",
        &["git", "commit"],
        "body",
    )];
    let loader = SkillLoader {
        skills,
        search_paths: Vec::new(),
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let manager = SkillManager::new(loader);

    assert_eq!(
        manager.match_skill("I need to GIT push"),
        Some("git".to_string())
    );
    assert_eq!(
        manager.match_skill("Let me Commit changes"),
        Some("git".to_string())
    );
}

#[test]
fn skill_manager_match_skill_no_match() {
    let skills = vec![make_skill("git", "Git operations", &["git"], "body")];
    let loader = SkillLoader {
        skills,
        search_paths: Vec::new(),
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let manager = SkillManager::new(loader);
    assert_eq!(manager.match_skill("write a python script"), None);
}

#[test]
fn skill_manager_match_skill_first_wins() {
    let skills = vec![
        make_skill("git", "Git operations", &["code"], "body"),
        make_skill("test", "Testing", &["code"], "body"),
    ];
    let loader = SkillLoader {
        skills,
        search_paths: Vec::new(),
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let manager = SkillManager::new(loader);
    // First matching skill wins
    assert_eq!(
        manager.match_skill("write some code"),
        Some("git".to_string())
    );
}

#[test]
fn skill_manager_load_reference_level2() {
    let dir = tempfile::tempdir().expect("temp dir");
    let skill_dir = dir.path().join("my-skill");
    fs::create_dir(&skill_dir).expect("create dir");

    let skill_content = r#"---
name: my-skill
description: A skill with references
triggers:
  - reference
---

# My Skill

See `template.txt` for the template.
"#;
    fs::write(skill_dir.join("SKILL.md"), skill_content).expect("write SKILL.md");
    fs::write(skill_dir.join("template.txt"), "Hello {{name}}").expect("write template");

    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skill_dir.clone()],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");

    let mut manager = SkillManager::new(loader);
    manager.load_skill("my-skill").expect("load skill");

    let ref_content = manager
        .load_reference("my-skill", "template.txt")
        .expect("load reference");
    assert_eq!(ref_content, "Hello {{name}}");
}

#[test]
fn skill_manager_load_reference_skill_not_loaded() {
    let loader = SkillLoader {
        skills: Vec::new(),
        search_paths: Vec::new(),
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let manager = SkillManager::new(loader);
    let result = manager.load_reference("not-loaded", "file.txt");
    assert!(result.is_err());
}

#[test]
fn skill_manager_load_reference_file_not_found() {
    let skill = make_skill("my-skill", "desc", &["x"], "body");
    let loader = SkillLoader {
        skills: vec![skill],
        search_paths: Vec::new(),
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let mut manager = SkillManager::new(loader);
    manager.load_skill("my-skill").expect("load");

    // Source path is /tmp/skills/my-skill/SKILL.md, nonexistent.txt won't exist
    let result = manager.load_reference("my-skill", "nonexistent.txt");
    assert!(result.is_err());
}

#[test]
fn skill_disclosure_enum_variants() {
    let l0 = SkillDisclosure::Level0;
    let l1 = SkillDisclosure::Level1;
    let l2 = SkillDisclosure::Level2;

    assert_eq!(l0, SkillDisclosure::Level0);
    assert_ne!(l0, l1);
    assert_ne!(l1, l2);
    assert_ne!(l0, l2);
}

// -----------------------------------------------------------------------
// Symlink discovery tests
// -----------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn discover_ignores_symlinked_directory_by_default() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("temp dir");
    let skills_dir = dir.path().join("skills");
    let real_skill_dir = dir.path().join("real-skill-dir");
    let linked_skill_dir = skills_dir.join("linked-skill");

    fs::create_dir_all(&skills_dir).expect("create skills dir");
    fs::create_dir_all(&real_skill_dir).expect("create real skill dir");
    fs::write(real_skill_dir.join("SKILL.md"), valid_skill_content()).expect("write SKILL.md");

    symlink(&real_skill_dir, &linked_skill_dir).expect("create symlink");

    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let skills = loader.discover().expect("discovery should succeed");
    assert_eq!(
        skills.len(),
        0,
        "symlinked directory not followed by default"
    );
}

#[cfg(unix)]
#[test]
fn discover_follows_symlinked_directory_when_enabled() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("temp dir");
    let skills_dir = dir.path().join("skills");
    let real_skill_dir = dir.path().join("real-skill-dir");
    let linked_skill_dir = skills_dir.join("linked-skill");

    fs::create_dir_all(&skills_dir).expect("create skills dir");
    fs::create_dir_all(&real_skill_dir).expect("create real skill dir");
    fs::write(real_skill_dir.join("SKILL.md"), valid_skill_content()).expect("write SKILL.md");

    symlink(&real_skill_dir, &linked_skill_dir).expect("create symlink");

    let policy = SkillDiscoveryPolicy {
        follow_directory_links: true,
        external_target_policy: ExternalTargetPolicy::AllowAnyReadable,
        ..Default::default()
    };

    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };

    let skills = loader.discover().expect("discovery should succeed");
    assert_eq!(skills.len(), 1, "skill found through symlinked directory");
    assert_eq!(skills[0].name, "test-skill");
}

#[cfg(unix)]
#[test]
fn discover_follows_symlinked_skills_root() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("temp dir");
    let real_root = dir.path().join("real-root");
    let linked_root = dir.path().join("linked-root");

    fs::create_dir_all(real_root.join("my-skill")).expect("create skill dir");
    fs::write(
        real_root.join("my-skill").join("SKILL.md"),
        "---\nname: symlinked-skill\ndescription: Found via symlink\ntriggers:\n  - sym\n---\n\nBody.\n",
    )
    .expect("write SKILL.md");

    symlink(&real_root, &linked_root).expect("create symlink to skills root");

    let policy = SkillDiscoveryPolicy {
        follow_directory_links: true,
        external_target_policy: ExternalTargetPolicy::AllowAnyReadable,
        ..Default::default()
    };

    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![linked_root],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };

    let skills = loader.discover().expect("discovery should succeed");
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].name, "symlinked-skill");
}

#[test]
fn discover_finds_regular_nested_skill() {
    let dir = tempfile::tempdir().expect("temp dir");
    let skills_dir = dir.path().join("skills");
    let nested_dir = skills_dir.join("level1").join("level2");
    fs::create_dir_all(&nested_dir).expect("create nested");

    let skill_content = "---\nname: deeply-nested\ndescription: In nested dirs\ntriggers:\n  - nested\n---\n\nBody.\n";
    fs::write(nested_dir.join("SKILL.md"), skill_content).expect("write nested SKILL.md");

    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };

    let skills = loader.discover().expect("discovery should succeed");
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].name, "deeply-nested");
}

#[test]
fn test_shared_skills_disabled_when_explicitly_off() {
    let dir = tempfile::tempdir().expect("operation should succeed");
    let loader = SkillLoader::for_workspace_with_options(dir.path(), false);
    // ~/.agents/skills should NOT be in search paths when explicitly disabled
    assert!(
        !loader
            .search_paths
            .iter()
            .any(|p| p.to_string_lossy().contains(".agents/skills"))
    );
}

#[test]
fn test_shared_skills_enabled_adds_path() {
    let home = home_dir().expect("home dir required");
    let shared_path = home.join(".agents").join("skills");
    fs::create_dir_all(&shared_path).expect("operation should succeed");

    let dir = tempfile::tempdir().expect("operation should succeed");
    let loader = SkillLoader::for_workspace_with_options(dir.path(), true);
    assert!(
        loader.search_paths.iter().any(|p| p == &shared_path),
        "~/.agents/skills should be in search paths when enabled"
    );

    let last = loader
        .search_paths
        .last()
        .expect("operation should succeed");
    assert_eq!(last, &shared_path);
}

#[test]
fn test_dedup_project_shadows_shared() {
    let home = home_dir().expect("home dir required");
    let shared_path = home.join(".agents").join("skills").join("dedup-test");
    let project_skills = tempfile::tempdir().expect("operation should succeed");
    let proj_skills_dir = project_skills.path().join(".talos/skills/dup-skill");

    fs::create_dir_all(&shared_path).expect("operation should succeed");
    fs::create_dir_all(&proj_skills_dir).expect("operation should succeed");

    fs::write(
        shared_path.join("SKILL.md"),
        "---\nname: dup-skill\ndescription: Shared version\ntriggers:\n  - dup\n---\n\nShared body.\n",
    )
    .expect("operation should succeed");

    fs::write(
        proj_skills_dir.join("SKILL.md"),
        "---\nname: dup-skill\ndescription: Project version\ntriggers:\n  - dup\n---\n\nProject body.\n",
    )
    .expect("operation should succeed");

    let mut loader = SkillLoader::for_workspace_with_options(project_skills.path(), true);
    loader.discover().expect("operation should succeed");

    let dup_skills: Vec<_> = loader
        .skills
        .iter()
        .filter(|s| s.name == "dup-skill")
        .collect();
    assert_eq!(
        dup_skills.len(),
        1,
        "dup-skill should appear exactly once after dedup"
    );
    assert_eq!(dup_skills[0].description, "Project version");
    assert_eq!(dup_skills[0].source, SkillSource::Project);

    let _ = fs::remove_dir_all(&shared_path);
}

#[test]
fn test_skill_source_tagged_correctly() {
    let project_skills = tempfile::tempdir().expect("operation should succeed");
    let proj_skills_dir = project_skills.path().join(".talos/skills/proj-skill");
    let shared_dir = project_skills.path().join("shared-skills");

    fs::create_dir_all(&proj_skills_dir).expect("operation should succeed");
    fs::create_dir_all(&shared_dir).expect("operation should succeed");

    fs::write(
        shared_dir.join("SKILL.md"),
        "---\nname: shared-only\ndescription: From shared\ntriggers:\n  - shared\n---\n\nShared.\n",
    )
    .expect("operation should succeed");

    fs::write(
        proj_skills_dir.join("SKILL.md"),
        "---\nname: proj-only\ndescription: From project\ntriggers:\n  - proj\n---\n\nProject.\n",
    )
    .expect("operation should succeed");

    // Manually construct loader with known search paths to avoid home-dir race
    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![proj_skills_dir.clone(), shared_dir.clone()],
        discover_shared: true,
        workspace_root: Some(project_skills.path().to_path_buf()),
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("operation should succeed");

    // Project skill should be Project source
    let proj_skill = loader
        .skills
        .iter()
        .find(|s| s.name == "proj-only")
        .expect("operation should succeed");
    assert_eq!(proj_skill.source, SkillSource::Project);

    // Second path skill should be Parent source (not in ~/.talos/skills or workspace .talos/skills)
    let shared_skill = loader
        .skills
        .iter()
        .find(|s| s.name == "shared-only")
        .expect("operation should succeed");
    assert_eq!(shared_skill.source, SkillSource::Parent);

    // Verify index propagates source
    let index = loader.get_index();
    let proj_idx = index
        .iter()
        .find(|e| e.name == "proj-only")
        .expect("operation should succeed");
    assert_eq!(proj_idx.source, SkillSource::Project);
    let shared_idx = index
        .iter()
        .find(|e| e.name == "shared-only")
        .expect("operation should succeed");
    assert_eq!(shared_idx.source, SkillSource::Parent);
}

#[test]
fn test_skill_source_display() {
    assert_eq!(SkillSource::Project.to_string(), "project");
    assert_eq!(SkillSource::Parent.to_string(), "parent");
    assert_eq!(SkillSource::UserGlobal.to_string(), "user");
    assert_eq!(SkillSource::Shared.to_string(), "shared");
}

#[test]
fn test_shared_skills_not_loaded_when_explicitly_disabled() {
    let project_skills = tempfile::tempdir().expect("operation should succeed");
    let loader = SkillLoader::for_workspace_with_options(project_skills.path(), false);
    assert!(
        !loader
            .search_paths
            .iter()
            .any(|p| p.to_string_lossy().contains(".agents/skills")),
        "~/.agents/skills should not be in search paths when explicitly disabled"
    );
}

// -----------------------------------------------------------------------
// Policy, dedup, and observability tests
// -----------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn same_physical_skill_via_two_aliases_is_loaded_once() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("temp dir");
    let skills_dir = dir.path().join("skills");
    let real_dir = dir.path().join("real-skill");
    fs::create_dir_all(&skills_dir).expect("create skills");
    fs::create_dir_all(&real_dir).expect("create real");
    fs::write(real_dir.join("SKILL.md"), valid_skill_content()).expect("write");

    symlink(&real_dir, skills_dir.join("alias-a")).expect("alias-a");
    symlink(&real_dir, skills_dir.join("alias-b")).expect("alias-b");

    let policy = SkillDiscoveryPolicy {
        follow_directory_links: true,
        external_target_policy: ExternalTargetPolicy::AllowAnyReadable,
        ..Default::default()
    };
    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    let count = loader
        .skills
        .iter()
        .filter(|s| s.name == "test-skill")
        .count();
    assert_eq!(count, 1, "same physical skill via two aliases loaded once");
}

#[cfg(unix)]
#[test]
fn broken_symlink_produces_warning() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("temp dir");
    let skills_dir = dir.path().join("skills");
    fs::create_dir_all(&skills_dir).expect("create skills");
    fs::write(skills_dir.join("SKILL.md"), valid_skill_content()).expect("write root skill");

    let broken = skills_dir.join("broken-link");
    symlink("/nonexistent/target", &broken).expect("create broken symlink");

    let policy = SkillDiscoveryPolicy {
        follow_directory_links: true,
        external_target_policy: ExternalTargetPolicy::AllowAnyReadable,
        ..Default::default()
    };
    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    assert!(
        loader
            .discovery_warnings
            .iter()
            .any(|w| w.kind == SkillDiscoveryWarningKind::BrokenLink),
        "broken link produces BrokenLink warning: {:?}",
        loader.discovery_warnings
    );
}

#[test]
fn non_adjacent_duplicate_names_follow_first_wins() {
    let dir1 = tempfile::tempdir().expect("temp");
    let dir2 = tempfile::tempdir().expect("temp");
    let dir3 = tempfile::tempdir().expect("temp");
    let s1 = dir1.path();
    let s2 = dir2.path();
    let s3 = dir3.path();

    fs::write(
        s1.join("SKILL.md"),
        "---\nname: dup\ndescription: Version 1\ntriggers:\n  - dup\n---\n\nBody 1.\n",
    )
    .expect("write");
    fs::write(
        s2.join("SKILL.md"),
        "---\nname: other\ndescription: Unique\ntriggers:\n  - other\n---\n\nBody.\n",
    )
    .expect("write");
    fs::write(
        s3.join("SKILL.md"),
        "---\nname: dup\ndescription: Version 2\ntriggers:\n  - dup\n---\n\nBody 2.\n",
    )
    .expect("write");

    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![s1.to_path_buf(), s2.to_path_buf(), s3.to_path_buf()],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");

    let dup_count = loader.skills.iter().filter(|s| s.name == "dup").count();
    assert_eq!(dup_count, 1, "non-adjacent duplicate removed globally");
    let dup = loader
        .skills
        .iter()
        .find(|s| s.name == "dup")
        .expect("operation should succeed");
    assert_eq!(dup.description, "Version 1", "first occurrence wins");
}

#[cfg(unix)]
#[test]
fn external_target_denied_by_default_when_following_links() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("temp");
    let skills_dir = dir.path().join("skills");
    let outside = dir.path().join("outside");
    fs::create_dir_all(&skills_dir).expect("create skills");
    fs::create_dir_all(&outside).expect("create outside");
    fs::write(outside.join("SKILL.md"), valid_skill_content()).expect("write");
    symlink(&outside, skills_dir.join("external-link")).expect("symlink");

    let policy = SkillDiscoveryPolicy {
        follow_directory_links: true,
        external_target_policy: ExternalTargetPolicy::DenyOutsideSearchRoot,
        ..Default::default()
    };
    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    assert_eq!(loader.skills.len(), 0, "external target denied");
    assert!(
        loader
            .discovery_warnings
            .iter()
            .any(|w| w.kind == SkillDiscoveryWarningKind::ExternalTargetDenied),
        "ExternalTargetDenied warning present"
    );
}

#[test]
fn entry_budget_uses_unique_skill_names() {
    let dir = tempfile::tempdir().expect("temp");
    let skills_dir = dir.path().join("skills");
    fs::create_dir_all(&skills_dir).expect("create");
    for i in 0..50 {
        let sub = skills_dir.join(format!("skill-{i:03}"));
        fs::create_dir_all(&sub).expect("create sub");
        let content = format!(
            "---\nname: skill-{i:03}\ndescription: Skill number {i}\ntriggers:\n  - skill{i}\n---\n\nBody {i}.\n"
        );
        fs::write(sub.join("SKILL.md"), content).expect("write");
    }

    let policy = SkillDiscoveryPolicy {
        max_entries: 10,
        ..Default::default()
    };
    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    assert!(
        loader
            .discovery_warnings
            .iter()
            .any(|w| w.kind == SkillDiscoveryWarningKind::EntryBudgetReached),
        "EntryBudgetReached warning present: {:?}",
        loader.discovery_warnings
    );
    let budget_count = loader
        .discovery_warnings
        .iter()
        .filter(|w| w.kind == SkillDiscoveryWarningKind::EntryBudgetReached)
        .count();
    assert_eq!(budget_count, 1, "EntryBudgetReached emitted exactly once");
    assert!(
        loader.skills.len() < 50,
        "not all skills found due to budget, got {}",
        loader.skills.len()
    );
    assert!(
        loader.skills.iter().all(|s| s.name.starts_with("skill-")),
        "all discovered skills have unique names"
    );
}

#[test]
fn depth_limit_bounds_traversal() {
    let dir = tempfile::tempdir().expect("temp");
    let skills_dir = dir.path().join("skills");
    let mut current = skills_dir.clone();
    fs::create_dir_all(&current).expect("create");

    for _ in 0..5 {
        let next = current.join("nested");
        fs::create_dir_all(&next).expect("create nested");
        current = next;
    }
    fs::write(current.join("SKILL.md"), valid_skill_content()).expect("write");

    let policy = SkillDiscoveryPolicy {
        max_depth: 3,
        ..Default::default()
    };
    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    assert_eq!(loader.skills.len(), 0, "skill beyond max_depth not found");

    let policy2 = SkillDiscoveryPolicy {
        max_depth: 6,
        ..Default::default()
    };
    loader.discovery_policy = policy2;
    loader.discover().expect("discover deeper");
    assert_eq!(loader.skills.len(), 1, "skill within max_depth found");
}

#[test]
fn default_policy_does_not_follow_links() {
    let p = SkillDiscoveryPolicy::default();
    assert!(!p.follow_directory_links, "default does not follow links");
    assert_eq!(p.max_depth, 32, "default max_depth is 32");
    assert_eq!(p.max_entries, 10_000, "default max_entries is 10000");
}

#[cfg(unix)]
#[test]
fn symlink_cycle_reports_link_loop_and_discovers_once() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("temp");
    let skills_dir = dir.path().join("skills");
    let sub_a = skills_dir.join("a");
    let sub_b = skills_dir.join("b");

    fs::create_dir_all(&sub_a).expect("create a");
    fs::create_dir_all(&sub_b).expect("create b");
    symlink(&sub_b, sub_a.join("link-to-b")).expect("a->b");
    symlink(&sub_a, sub_b.join("link-to-a")).expect("b->a");
    fs::write(sub_a.join("SKILL.md"), valid_skill_content()).expect("write skill");

    let policy = SkillDiscoveryPolicy {
        follow_directory_links: true,
        external_target_policy: ExternalTargetPolicy::AllowAnyReadable,
        ..Default::default()
    };
    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    let count = loader
        .skills
        .iter()
        .filter(|s| s.name == "test-skill")
        .count();
    assert_eq!(count, 1, "skill found exactly once despite cycle");
    assert!(
        loader
            .discovery_warnings
            .iter()
            .any(|w| w.kind == SkillDiscoveryWarningKind::LinkLoop),
        "LinkLoop warning present: {:?}",
        loader.discovery_warnings
    );
    assert!(
        !loader
            .discovery_warnings
            .iter()
            .any(|w| w.kind == SkillDiscoveryWarningKind::EntryBudgetReached),
        "cycle must not trigger entry budget"
    );
}

// -----------------------------------------------------------------------
// Real two-link chain, root-link policy, pre-descent rejection,
// alias subtree single descent, root-level SKILL.md, direct file links,
// depth boundary, global budget
// -----------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn discover_follows_actual_two_link_chain() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("temp");
    let skills_dir = dir.path().join("skills");
    let intermediate = dir.path().join("intermediate");
    let final_dir = dir.path().join("final");

    fs::create_dir_all(&skills_dir).expect("create skills");
    fs::create_dir_all(&intermediate).expect("create intermediate");
    fs::create_dir_all(&final_dir).expect("create final");
    fs::write(final_dir.join("SKILL.md"), valid_skill_content()).expect("write SKILL.md");

    symlink(&intermediate, skills_dir.join("link-a")).expect("link-a -> intermediate");
    symlink(&final_dir, intermediate.join("link-b")).expect("link-b -> final");

    let policy = SkillDiscoveryPolicy {
        follow_directory_links: true,
        external_target_policy: ExternalTargetPolicy::AllowAnyReadable,
        ..Default::default()
    };
    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    assert_eq!(
        loader.skills.len(),
        1,
        "skill found through real two-link chain"
    );
    assert_eq!(loader.skills[0].name, "test-skill");
}

#[cfg(unix)]
#[test]
fn default_policy_rejects_symlinked_project_skills_root() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("temp");
    let real_root = dir.path().join("real-root");
    let linked_root = dir.path().join("linked-root");

    fs::create_dir_all(real_root.join("my-skill")).expect("create skill dir");
    fs::write(
        real_root.join("my-skill").join("SKILL.md"),
        "---\nname: linked-root-skill\ndescription: Found via symlink\ntriggers:\n  - sym\n---\n\nBody.\n",
    )
    .expect("write SKILL.md");
    symlink(&real_root, &linked_root).expect("create symlink to skills root");

    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![linked_root],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    assert_eq!(
        loader.skills.len(),
        0,
        "symlinked project root rejected by default"
    );
    assert!(
        loader
            .discovery_warnings
            .iter()
            .any(|w| w.kind == SkillDiscoveryWarningKind::RootLinkDenied),
        "RootLinkDenied warning present: {:?}",
        loader.discovery_warnings
    );
}

#[cfg(unix)]
#[test]
fn default_policy_rejects_symlinked_shared_skills_root() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("temp");
    let real_shared = dir.path().join("real-shared");
    let linked_shared = dir.path().join(".agents").join("skills");

    fs::create_dir_all(&real_shared).expect("create real shared");
    fs::create_dir_all(real_shared.join("shared-skill")).expect("create skill dir");
    fs::write(
        real_shared.join("shared-skill").join("SKILL.md"),
        "---\nname: shared-via-symlink\ndescription: Shared\ntriggers:\n  - sym\n---\n\nBody.\n",
    )
    .expect("write SKILL.md");
    fs::create_dir_all(dir.path().join(".agents")).expect("create .agents");
    symlink(&real_shared, &linked_shared).expect("create symlink to shared root");

    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![linked_shared],
        discover_shared: true,
        workspace_root: None,
        discovery_policy: SkillDiscoveryPolicy::default(),
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    assert_eq!(
        loader.skills.len(),
        0,
        "symlinked shared root rejected by default"
    );
    assert!(
        loader
            .discovery_warnings
            .iter()
            .any(|w| w.kind == SkillDiscoveryWarningKind::RootLinkDenied),
        "RootLinkDenied warning present even for shared root: {:?}",
        loader.discovery_warnings
    );
}

#[cfg(unix)]
#[test]
fn explicit_allow_any_permits_symlinked_root() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("temp");
    let real_root = dir.path().join("real-root");
    let linked_root = dir.path().join("linked-root");

    fs::create_dir_all(real_root.join("my-skill")).expect("create skill dir");
    fs::write(
        real_root.join("my-skill").join("SKILL.md"),
        "---\nname: allow-any-skill\ndescription: Allowed\ntriggers:\n  - allow\n---\n\nBody.\n",
    )
    .expect("write SKILL.md");
    symlink(&real_root, &linked_root).expect("create symlink to skills root");

    let policy = SkillDiscoveryPolicy {
        follow_directory_links: true,
        external_target_policy: ExternalTargetPolicy::AllowAnyReadable,
        ..Default::default()
    };
    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![linked_root],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    assert_eq!(
        loader.skills.len(),
        1,
        "symlinked root permitted with AllowAnyReadable"
    );
    assert_eq!(loader.skills[0].name, "allow-any-skill");
}

#[cfg(unix)]
#[test]
fn external_directory_is_rejected_before_descent() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("temp");
    let skills_dir = dir.path().join("skills");
    let outside = dir.path().join("outside");

    fs::create_dir_all(&skills_dir).expect("create skills");
    fs::create_dir_all(&outside).expect("create outside");
    fs::write(outside.join("SKILL.md"), "no frontmatter").expect("write invalid SKILL.md");
    for i in 0..20 {
        fs::create_dir_all(outside.join(format!("sub-{i}"))).expect("create sub");
    }
    symlink(&outside, skills_dir.join("external-link")).expect("symlink to outside");

    let policy = SkillDiscoveryPolicy {
        follow_directory_links: true,
        external_target_policy: ExternalTargetPolicy::DenyOutsideSearchRoot,
        max_entries: 5,
        ..Default::default()
    };
    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    assert!(
        loader
            .discovery_warnings
            .iter()
            .any(|w| w.kind == SkillDiscoveryWarningKind::ExternalTargetDenied),
        "ExternalTargetDenied present: {:?}",
        loader.discovery_warnings
    );
    assert!(
        !loader
            .discovery_warnings
            .iter()
            .any(|w| w.kind == SkillDiscoveryWarningKind::InvalidSkill),
        "external tree must not be traversed (no InvalidSkill): {:?}",
        loader.discovery_warnings
    );
    assert!(
        !loader
            .discovery_warnings
            .iter()
            .any(|w| w.kind == SkillDiscoveryWarningKind::EntryBudgetReached),
        "external tree must not consume budget: {:?}",
        loader.discovery_warnings
    );
    assert_eq!(loader.skills.len(), 0, "no skills from external tree");
}

#[cfg(unix)]
#[test]
fn canonical_alias_subtree_is_descended_once() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("temp");
    let skills_dir = dir.path().join("skills");
    let real_tree = dir.path().join("real-tree");

    fs::create_dir_all(&skills_dir).expect("create skills");
    fs::create_dir_all(&real_tree).expect("create real tree");
    fs::write(real_tree.join("SKILL.md"), valid_skill_content()).expect("write SKILL.md");
    symlink(&real_tree, skills_dir.join("alias-a")).expect("alias-a");
    symlink(&real_tree, skills_dir.join("alias-b")).expect("alias-b");

    let policy = SkillDiscoveryPolicy {
        follow_directory_links: true,
        external_target_policy: ExternalTargetPolicy::AllowAnyReadable,
        max_entries: 4,
        ..Default::default()
    };
    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    assert_eq!(loader.skills.len(), 1, "skill loaded once");
    assert!(
        !loader
            .discovery_warnings
            .iter()
            .any(|w| w.kind == SkillDiscoveryWarningKind::EntryBudgetReached),
        "physical subtree traversed once (budget not exhausted): {:?}",
        loader.discovery_warnings
    );
}

#[test]
fn root_level_skill_is_loaded_when_link_following_is_enabled() {
    let dir = tempfile::tempdir().expect("temp");
    let skills_dir = dir.path().join("skills");
    fs::create_dir_all(&skills_dir).expect("create skills");
    fs::write(skills_dir.join("SKILL.md"), valid_skill_content()).expect("write root SKILL.md");

    let policy = SkillDiscoveryPolicy {
        follow_directory_links: true,
        external_target_policy: ExternalTargetPolicy::DenyOutsideSearchRoot,
        ..Default::default()
    };
    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    assert_eq!(
        loader.skills.len(),
        1,
        "root-level SKILL.md parsed when link following enabled"
    );
    assert_eq!(loader.skills[0].name, "test-skill");
    let dup_count = loader
        .discovery_warnings
        .iter()
        .filter(|w| w.kind == SkillDiscoveryWarningKind::InvalidSkill)
        .count();
    assert_eq!(
        dup_count, 0,
        "no duplicate parse warning for root-level SKILL.md"
    );
}

#[cfg(unix)]
#[test]
fn external_skill_file_symlink_is_denied_before_read() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("temp");
    let skills_dir = dir.path().join("skills");
    let outside = dir.path().join("outside");

    fs::create_dir_all(&skills_dir).expect("create skills");
    fs::create_dir_all(&outside).expect("create outside");
    fs::write(outside.join("SKILL.md"), valid_skill_content()).expect("write external SKILL.md");
    symlink(outside.join("SKILL.md"), skills_dir.join("SKILL.md")).expect("file symlink");

    let policy = SkillDiscoveryPolicy {
        follow_directory_links: true,
        external_target_policy: ExternalTargetPolicy::DenyOutsideSearchRoot,
        ..Default::default()
    };
    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    assert_eq!(loader.skills.len(), 0, "external file symlink not read");
    assert!(
        loader
            .discovery_warnings
            .iter()
            .any(|w| w.kind == SkillDiscoveryWarningKind::ExternalTargetDenied),
        "ExternalTargetDenied for external file symlink: {:?}",
        loader.discovery_warnings
    );
}

#[cfg(unix)]
#[test]
fn same_skill_file_via_two_links_is_parsed_once() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("temp");
    let skills_dir = dir.path().join("skills");
    let real_dir = skills_dir.join("real");
    let link_a = skills_dir.join("link-a");
    let link_b = skills_dir.join("link-b");

    fs::create_dir_all(&real_dir).expect("create real");
    fs::write(real_dir.join("SKILL.md"), valid_skill_content()).expect("write SKILL.md");
    fs::create_dir_all(&link_a).expect("create link-a");
    fs::create_dir_all(&link_b).expect("create link-b");
    symlink(real_dir.join("SKILL.md"), link_a.join("SKILL.md")).expect("link-a SKILL.md");
    symlink(real_dir.join("SKILL.md"), link_b.join("SKILL.md")).expect("link-b SKILL.md");

    let policy = SkillDiscoveryPolicy {
        follow_directory_links: true,
        external_target_policy: ExternalTargetPolicy::AllowAnyReadable,
        ..Default::default()
    };
    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    let count = loader
        .skills
        .iter()
        .filter(|s| s.name == "test-skill")
        .count();
    assert_eq!(count, 1, "same skill file via two links parsed once");
}

#[cfg(unix)]
#[test]
fn internal_skill_file_symlink_is_followed_only_when_enabled() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().expect("temp");
    let skills_dir = dir.path().join("skills");
    fs::create_dir_all(&skills_dir).expect("create skills");
    fs::write(skills_dir.join("real.md"), valid_skill_content()).expect("write real");
    symlink(skills_dir.join("real.md"), skills_dir.join("SKILL.md"))
        .expect("symlink SKILL.md -> real.md");

    let policy_off = SkillDiscoveryPolicy {
        follow_directory_links: false,
        ..Default::default()
    };
    let mut loader_off = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir.clone()],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy_off,
        discovery_warnings: Vec::new(),
    };
    loader_off.discover().expect("discover off");
    assert_eq!(
        loader_off.skills.len(),
        0,
        "symlinked SKILL.md not read when follow=false"
    );
    assert!(
        loader_off
            .discovery_warnings
            .iter()
            .any(|w| w.kind == SkillDiscoveryWarningKind::RootLinkDenied),
        "RootLinkDenied for symlinked SKILL.md when follow=false: {:?}",
        loader_off.discovery_warnings
    );

    let policy_on = SkillDiscoveryPolicy {
        follow_directory_links: true,
        external_target_policy: ExternalTargetPolicy::AllowAnyReadable,
        ..Default::default()
    };
    let mut loader_on = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy_on,
        discovery_warnings: Vec::new(),
    };
    loader_on.discover().expect("discover on");
    assert_eq!(
        loader_on.skills.len(),
        1,
        "symlinked SKILL.md read when follow=true"
    );
    assert_eq!(loader_on.skills[0].name, "test-skill");
}

#[test]
fn skill_at_exact_max_depth_is_discovered() {
    let dir = tempfile::tempdir().expect("temp");
    let skills_dir = dir.path().join("skills");
    let nested = skills_dir.join("nested");
    fs::create_dir_all(&nested).expect("create nested");
    fs::write(nested.join("SKILL.md"), valid_skill_content()).expect("write SKILL.md");

    let policy = SkillDiscoveryPolicy {
        max_depth: 2,
        ..Default::default()
    };
    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    assert_eq!(
        loader.skills.len(),
        1,
        "skill at exact max_depth is discovered"
    );
}

#[test]
fn skill_beyond_max_depth_is_not_discovered() {
    let dir = tempfile::tempdir().expect("temp");
    let skills_dir = dir.path().join("skills");
    let nested = skills_dir.join("nested").join("nested");
    fs::create_dir_all(&nested).expect("create nested");
    fs::write(nested.join("SKILL.md"), valid_skill_content()).expect("write SKILL.md");

    let policy = SkillDiscoveryPolicy {
        max_depth: 2,
        ..Default::default()
    };
    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    assert_eq!(
        loader.skills.len(),
        0,
        "skill beyond max_depth not discovered"
    );
    assert!(
        loader
            .discovery_warnings
            .iter()
            .any(|w| w.kind == SkillDiscoveryWarningKind::DepthLimitReached),
        "DepthLimitReached warning present: {:?}",
        loader.discovery_warnings
    );
}

#[test]
fn depth_limit_produces_single_warning() {
    let dir = tempfile::tempdir().expect("temp");
    let skills_dir = dir.path().join("skills");
    let mut current = skills_dir.clone();
    fs::create_dir_all(&current).expect("create");
    for _ in 0..5 {
        let next = current.join("nested");
        fs::create_dir_all(&next).expect("create nested");
        current = next;
    }
    fs::write(current.join("SKILL.md"), valid_skill_content()).expect("write");

    let policy = SkillDiscoveryPolicy {
        max_depth: 2,
        ..Default::default()
    };
    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills_dir],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    let depth_count = loader
        .discovery_warnings
        .iter()
        .filter(|w| w.kind == SkillDiscoveryWarningKind::DepthLimitReached)
        .count();
    assert_eq!(
        depth_count, 1,
        "DepthLimitReached emitted exactly once per root"
    );
}

#[test]
fn entry_budget_is_global_across_search_roots() {
    let dir1 = tempfile::tempdir().expect("temp");
    let dir2 = tempfile::tempdir().expect("temp");
    let skills1 = dir1.path().join("skills");
    let skills2 = dir2.path().join("skills");
    fs::create_dir_all(&skills1).expect("create 1");
    fs::create_dir_all(&skills2).expect("create 2");
    for i in 0..20 {
        let sub = skills1.join(format!("skill-{i:03}"));
        fs::create_dir_all(&sub).expect("create sub");
        fs::write(
            sub.join("SKILL.md"),
            format!("---\nname: first-{i:03}\ndescription: First {i}\ntriggers:\n  - first{i}\n---\n\nBody.\n"),
        )
        .expect("write");
    }
    fs::write(
        skills2.join("SKILL.md"),
        "---\nname: sentinel-from-second-root\ndescription: Sentinel\ntriggers:\n  - sentinel\n---\n\nBody.\n",
    )
    .expect("write sentinel");

    let policy = SkillDiscoveryPolicy {
        max_entries: 5,
        ..Default::default()
    };
    let mut loader = SkillLoader {
        skills: Vec::new(),
        search_paths: vec![skills1, skills2],
        discover_shared: false,
        workspace_root: None,
        discovery_policy: policy,
        discovery_warnings: Vec::new(),
    };
    loader.discover().expect("discover");
    assert!(
        loader
            .discovery_warnings
            .iter()
            .any(|w| w.kind == SkillDiscoveryWarningKind::EntryBudgetReached),
        "EntryBudgetReached present"
    );
    assert!(
        !loader
            .skills
            .iter()
            .any(|s| s.name == "sentinel-from-second-root"),
        "later root sentinel must not be discovered after budget exhaustion"
    );
}
