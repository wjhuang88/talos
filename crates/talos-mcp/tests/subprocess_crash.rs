use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use talos_mcp::client::McpClientManager;
use talos_mcp::types::{McpClientConfig, McpServerLaunchConfig};
use talos_plugin::HookRegistry;

#[tokio::test]
async fn subprocess_crash_is_reported_and_other_servers_continue() {
    let fixture_path = build_fixture_binary();

    let config = McpClientConfig {
        servers: vec![
            McpServerLaunchConfig {
                name: "crash".to_string(),
                transport: "stdio".to_string(),
                command: "/usr/bin/false".to_string(),
                args: Vec::new(),
                env: HashMap::new(),
                cwd: None,
                ..McpServerLaunchConfig::default()
            },
            McpServerLaunchConfig {
                name: "ok".to_string(),
                transport: "stdio".to_string(),
                command: fixture_path.to_string_lossy().to_string(),
                args: Vec::new(),
                env: HashMap::from([("ECHO_PREFIX".to_string(), "ok".to_string())]),
                cwd: std::env::current_dir().ok(),
                ..McpServerLaunchConfig::default()
            },
        ],
    };

    let manager = McpClientManager::start(&config, Arc::new(HookRegistry::new()))
        .await
        .expect("manager should start with partial failure");

    assert!(
        manager
            .startup_failures()
            .iter()
            .any(|failure| failure.server == "crash"),
        "crash server failure should be reported"
    );

    let tools = manager.discover_tools().await;
    assert!(
        tools.iter().any(|tool| tool.name() == "mcp:ok:echo"),
        "healthy server tools should still be discoverable"
    );
}

fn build_fixture_binary() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");
    let source = workspace
        .join("crates")
        .join("talos-cli")
        .join("tests")
        .join("fixtures")
        .join("echo_mcp_server.rs");
    let output = std::env::temp_dir().join("talos_echo_mcp_server_fixture_bin");

    let status = std::process::Command::new("rustc")
        .arg("--edition=2024")
        .arg("-o")
        .arg(&output)
        .arg(source)
        .status()
        .expect("spawn rustc for fixture");
    assert!(status.success(), "fixture rustc compile failed");
    output
}
