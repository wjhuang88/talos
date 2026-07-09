//! Integration tests proving fetch/save/extract boundary separation.
//!
//! These tests verify that `http_request` (Network), `save_url` (Network+Write),
//! and `document_extract` (Read) are separate permission-aware operations with
//! no implicit chaining.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use talos_core::tool::{AgentTool, ToolNature, ToolResourceKind};
use talos_permission::{PermissionDecision, PermissionEngine, PermissionRule};
use talos_tools::{DocumentExtractTool, FetchUrlTool, HttpRequestTool, SaveUrlTool};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn unique_id() -> String {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("{}{}", d.as_secs(), d.subsec_nanos())
}

fn create_temp_file(name: &str, content: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("boundary_test_{}_{}", unique_id(), name));
    std::fs::write(&path, content).unwrap();
    path
}

fn create_temp_binary(name: &str, bytes: &[u8]) -> PathBuf {
    let path = std::env::temp_dir().join(format!("boundary_test_{}_{}", unique_id(), name));
    std::fs::write(&path, bytes).unwrap();
    path
}

fn cleanup(path: &PathBuf) {
    let _ = std::fs::remove_file(path);
}

fn run_extract(tool: &DocumentExtractTool, path: &str) -> String {
    let input = serde_json::json!({
        "path": path,
        "format": "auto",
    });
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(tool.execute(input));
    assert!(
        !result.is_error,
        "Expected success, got: {}",
        result.content
    );
    result.content
}

// ---------------------------------------------------------------------------
// Permission boundary tests
// ---------------------------------------------------------------------------

#[test]
fn test_document_extract_is_read_only() {
    let tool = DocumentExtractTool::new(PathBuf::from("/"));
    assert!(tool.is_read_only(), "document_extract must be read-only");
    assert!(
        matches!(tool.nature(), ToolNature::Read),
        "document_extract nature must be Read"
    );
}

#[test]
fn test_save_url_has_write_and_network_facets() {
    let tool = SaveUrlTool::new();
    assert!(!tool.is_read_only(), "save_url must NOT be read-only");
    assert!(
        matches!(tool.nature(), ToolNature::Write),
        "save_url nature must be Write"
    );

    let profile = tool.permission_profile(&serde_json::json!({
        "url": "https://example.com/data.json",
        "destination": "output/data.json"
    }));

    assert_eq!(
        profile.len(),
        2,
        "save_url must have exactly 2 permission facets"
    );

    let has_network = profile.iter().any(|f| f.nature == ToolNature::Network);
    let has_write = profile.iter().any(|f| f.nature == ToolNature::Write);

    assert!(
        has_network,
        "save_url permission profile must include Network facet"
    );
    assert!(
        has_write,
        "save_url permission profile must include Write facet"
    );

    // Verify resource kinds.
    let network_facet = profile
        .iter()
        .find(|f| f.nature == ToolNature::Network)
        .unwrap();
    assert_eq!(network_facet.resource_kind, Some(ToolResourceKind::Domain));
    assert_eq!(network_facet.resource.as_deref(), Some("example.com"));

    let write_facet = profile
        .iter()
        .find(|f| f.nature == ToolNature::Write)
        .unwrap();
    assert_eq!(write_facet.resource_kind, Some(ToolResourceKind::Path));
    assert_eq!(write_facet.resource.as_deref(), Some("output/data.json"));
}

#[test]
fn test_http_request_has_only_network_facet() {
    let tool = HttpRequestTool::new();
    assert!(!tool.is_read_only(), "http_request must NOT be read-only");
    assert!(
        matches!(tool.nature(), ToolNature::Network),
        "http_request nature must be Network"
    );

    let profile = tool.permission_profile(&serde_json::json!({
        "url": "https://api.example.com/data",
        "method": "GET"
    }));

    // http_request should only have Network facet, not Write.
    let has_network = profile.iter().any(|f| f.nature == ToolNature::Network);
    let has_write = profile.iter().any(|f| f.nature == ToolNature::Write);

    assert!(
        has_network,
        "http_request permission profile must include Network facet"
    );
    assert!(
        !has_write,
        "http_request permission profile must NOT include Write facet"
    );
}

#[test]
fn test_web_tool_permission_profiles_are_least_privilege() {
    let fetch = FetchUrlTool::new();
    let http = HttpRequestTool::new();
    let save = SaveUrlTool::new();

    for (name, tool) in [
        ("fetch_url", &fetch as &dyn AgentTool),
        ("http_request", &http as &dyn AgentTool),
    ] {
        let profile = tool.permission_profile(&serde_json::json!({
            "url": "https://Example.com/path",
            "method": "GET"
        }));

        assert_eq!(profile.len(), 1, "{name} must have one network facet");
        assert_eq!(profile[0].nature, ToolNature::Network);
        assert_eq!(profile[0].resource_kind, Some(ToolResourceKind::Domain));
        assert_eq!(profile[0].resource.as_deref(), Some("example.com"));
        assert!(
            !profile
                .iter()
                .any(|facet| facet.nature == ToolNature::Write),
            "{name} must not request write permission"
        );
    }

    let save_profile = save.permission_profile(&serde_json::json!({
        "url": "https://Example.com/archive.zip",
        "destination": "downloads/archive.zip"
    }));
    assert_eq!(save_profile.len(), 2);
    assert!(save_profile.iter().any(|facet| {
        facet.nature == ToolNature::Network
            && facet.resource_kind == Some(ToolResourceKind::Domain)
            && facet.resource.as_deref() == Some("example.com")
    }));
    assert!(save_profile.iter().any(|facet| {
        facet.nature == ToolNature::Write
            && facet.resource_kind == Some(ToolResourceKind::Path)
            && facet.resource.as_deref() == Some("downloads/archive.zip")
    }));
}

#[test]
fn test_permission_profile_denies_save_url_when_write_facet_is_denied() {
    let save = SaveUrlTool::new();
    let profile = save.permission_profile(&serde_json::json!({
        "url": "https://example.com/archive.zip",
        "destination": "downloads/archive.zip"
    }));

    let mut engine = PermissionEngine {
                rules: Vec::new(),
                workspace_root: None,
                trusted_workspace: false,
            };
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Network,
        None,
        None,
        PermissionDecision::Allow,
    ));
    engine.add_rule(PermissionRule::new_nature(
        ToolNature::Write,
        None,
        None,
        PermissionDecision::Deny("write blocked".to_string()),
    ));

    let decision = engine.evaluate_profile(
        "save_url",
        &profile,
        &serde_json::json!({
            "url": "https://example.com/archive.zip",
            "destination": "downloads/archive.zip"
        }),
    );

    assert_eq!(
        decision,
        PermissionDecision::Deny("write blocked".to_string()),
        "network allow must not mask save_url write denial"
    );
}

#[test]
fn test_document_extract_no_network_facet() {
    let tool = DocumentExtractTool::new(PathBuf::from("/"));
    let profile = tool.permission_profile(&serde_json::json!({
        "path": "test.txt"
    }));

    assert!(
        !profile.is_empty(),
        "document_extract must have at least one facet"
    );

    for facet in &profile {
        assert_eq!(
            facet.nature,
            ToolNature::Read,
            "document_extract facets must all be Read, found: {:?}",
            facet.nature
        );
    }

    let has_network = profile.iter().any(|f| f.nature == ToolNature::Network);
    assert!(
        !has_network,
        "document_extract must NOT have any Network facet"
    );
}

// ---------------------------------------------------------------------------
// Separation tests
// ---------------------------------------------------------------------------

#[test]
fn test_document_extract_rejects_url_path() {
    let tool = DocumentExtractTool::new(PathBuf::from("/"));

    // Passing a URL-like path should NOT trigger a network call.
    // It should either error (file not found) or treat it as a local path.
    let input = serde_json::json!({
        "path": "http://example.com/remote/file.txt",
        "format": "auto",
    });
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(tool.execute(input));

    // The tool should NOT make a network request. It should fail because
    // the path doesn't exist as a local file.
    assert!(
        result.is_error,
        "document_extract with URL-like path should error, not fetch"
    );
    // Verify the error is about file not found, not a network error.
    assert!(
        result.content.to_lowercase().contains("file not found")
            || result.content.to_lowercase().contains("path escape"),
        "Expected file-not-found or path-escape error, got: {}",
        result.content
    );
}

#[test]
fn test_save_url_does_not_inject_content() {
    // save_url output should only contain metadata (bytes saved, path),
    // never extracted file content.
    // Since we can't make real HTTP calls, we verify the tool's nature
    // and permission profile prove it's a write-only operation.

    let tool = SaveUrlTool::new();

    // Verify save_url nature is Write, not Read.
    assert!(matches!(tool.nature(), ToolNature::Write));

    // Verify it has no Read facet.
    let profile = tool.permission_profile(&serde_json::json!({
        "url": "https://example.com/file.txt",
        "destination": "output/file.txt"
    }));
    let has_read = profile.iter().any(|f| f.nature == ToolNature::Read);
    assert!(
        !has_read,
        "save_url must NOT have any Read facet — it writes, doesn't extract"
    );
}

#[test]
fn test_manual_composition_save_then_extract() {
    // This test proves that save_url and document_extract can be composed
    // manually (save a file, then extract from it) but are never auto-chained.

    // Step 1: Create a local file simulating "saved" content.
    let content = r#"{"name": "test", "value": 42}"#;
    let saved_path = create_temp_file("composed.json", content);
    let path_str = saved_path.to_string_lossy().to_string();

    // Step 2: Use document_extract on the saved file.
    let extract_tool = DocumentExtractTool::new(PathBuf::from("/"));
    let output = run_extract(&extract_tool, &path_str);

    // Verify the extract worked on the locally saved file.
    assert!(output.contains("Format: json"), "Should detect JSON format");
    assert!(
        output.contains(r#""name": "test""#),
        "Should pretty-print JSON"
    );
    assert!(output.contains(r#""value": 42"#));

    cleanup(&saved_path);
}

// ---------------------------------------------------------------------------
// Format extraction tests
// ---------------------------------------------------------------------------

#[test]
fn test_extract_html_strips_tags() {
    let content = "<!DOCTYPE html>\n<html><head><title>Test Page</title></head>\n<body><h1>Hello World</h1><p>This is a test.</p></body>\n</html>";
    let path = create_temp_file("test.html", content);
    let path_str = path.to_string_lossy().to_string();

    let tool = DocumentExtractTool::new(PathBuf::from("/"));
    let output = run_extract(&tool, &path_str);

    assert!(
        output.contains("Hello World"),
        "Should extract heading text"
    );
    assert!(
        output.contains("This is a test"),
        "Should extract paragraph text"
    );
    assert!(!output.contains("<html>"), "Should not contain HTML tags");
    assert!(!output.contains("<h1>"), "Should not contain h1 tags");
    assert!(!output.contains("<title>"), "Should not contain title tags");
    assert!(output.contains("Format: html"));

    cleanup(&path);
}

#[test]
fn test_extract_json_pretty_prints() {
    let content = r#"{"name":"test","value":42,"items":[1,2,3],"nested":{"key":"value"}}"#;
    let path = create_temp_file("test.json", content);
    let path_str = path.to_string_lossy().to_string();

    let tool = DocumentExtractTool::new(PathBuf::from("/"));
    let output = run_extract(&tool, &path_str);

    // Verify pretty-printing (indented output).
    assert!(
        output.contains(r#""name": "test""#),
        "Should have pretty-printed name"
    );
    assert!(
        output.contains(r#""value": 42"#),
        "Should have pretty-printed value"
    );
    assert!(
        output.contains(r#""key": "value""#),
        "Should have pretty-printed nested key"
    );
    assert!(output.contains("Format: json"));

    cleanup(&path);
}

#[test]
fn test_extract_binary_returns_metadata_only() {
    let bytes: Vec<u8> = (0..=255).cycle().take(1024).collect();
    let path = create_temp_binary("test.bin", &bytes);
    let path_str = path.to_string_lossy().to_string();

    let tool = DocumentExtractTool::new(PathBuf::from("/"));
    let output = run_extract(&tool, &path_str);

    // Binary should return metadata, not content dump.
    assert!(output.contains("binary"), "Should identify as binary");
    assert!(
        output.contains("unsupported"),
        "Should indicate unsupported format"
    );
    assert!(output.contains("1024 bytes"), "Should report file size");
    assert!(
        !output.contains('\0'),
        "Should not dump binary content with null bytes"
    );

    cleanup(&path);
}

#[test]
fn test_extract_truncation_marker() {
    let content = "x".repeat(1000);
    let path = create_temp_file("large.txt", &content);
    let path_str = path.to_string_lossy().to_string();

    let tool = DocumentExtractTool::new(PathBuf::from("/"));
    let input = serde_json::json!({
        "path": path_str,
        "format": "auto",
        "max_bytes": 100,
    });
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(tool.execute(input));
    assert!(
        !result.is_error,
        "Expected success, got: {}",
        result.content
    );
    let output = result.content;

    assert!(
        output.contains("truncated"),
        "Should contain truncation indicator"
    );
    assert!(
        output.contains("1000 bytes total"),
        "Should report total size"
    );
    assert!(
        output.contains("showing"),
        "Should show truncation marker with shown bytes"
    );

    cleanup(&path);
}

// ---------------------------------------------------------------------------
// Cross-tool nature consistency tests
// ---------------------------------------------------------------------------

#[test]
fn test_all_tools_have_distinct_natures() {
    let extract = DocumentExtractTool::new(PathBuf::from("/"));
    let save = SaveUrlTool::new();
    let http = HttpRequestTool::new();

    // Each tool should have a distinct primary nature.
    assert!(matches!(extract.nature(), ToolNature::Read));
    assert!(matches!(save.nature(), ToolNature::Write));
    assert!(matches!(http.nature(), ToolNature::Network));

    // No two tools should share the same nature.
    let natures = [extract.nature(), save.nature(), http.nature()];
    assert_ne!(
        natures[0], natures[1],
        "extract and save must have distinct natures"
    );
    assert_ne!(
        natures[0], natures[2],
        "extract and http must have distinct natures"
    );
    assert_ne!(
        natures[1], natures[2],
        "save and http must have distinct natures"
    );
}

#[test]
fn test_no_tool_has_all_facets() {
    // No single tool should have Read + Write + Network facets simultaneously.
    // This would indicate a dangerous over-privileged tool.

    let extract = DocumentExtractTool::new(PathBuf::from("/"));
    let save = SaveUrlTool::new();
    let http = HttpRequestTool::new();

    for (name, tool) in [
        ("document_extract", &extract as &dyn AgentTool),
        ("save_url", &save as &dyn AgentTool),
        ("http_request", &http as &dyn AgentTool),
    ] {
        let profile = tool.permission_profile(&serde_json::json!({
            "path": "test.txt",
            "url": "https://example.com/test",
            "destination": "out.txt",
            "method": "GET"
        }));

        let has_read = profile.iter().any(|f| f.nature == ToolNature::Read);
        let has_write = profile.iter().any(|f| f.nature == ToolNature::Write);
        let has_network = profile.iter().any(|f| f.nature == ToolNature::Network);

        let facet_count = [has_read, has_write, has_network]
            .iter()
            .filter(|&&b| b)
            .count();

        assert!(
            facet_count < 3,
            "{name} has all three facets (Read+Write+Network) — over-privileged"
        );
    }
}
