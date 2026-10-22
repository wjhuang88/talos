//! The unit-test acceptance entry must not enable the production Windows library.
#![cfg(all(windows, feature = "git"))]

use talos_core::tool::{AgentTool, ToolProvenance};
use talos_tools::git::{GitAddTool, GitCheckoutTool, GitCommitTool, GitPullTool, GitPushTool};

#[test]
fn production_windows_preparation_stays_closed_before_repository_discovery() {
    let root = tempfile::tempdir().expect("fixture");
    // Deliberately not a repository: the platform gate must win before discovery.
    let cases: Vec<(Box<dyn AgentTool>, serde_json::Value)> = vec![
        (
            Box::new(GitAddTool::new(root.path().to_owned())),
            serde_json::json!({"paths": ["file"]}),
        ),
        (
            Box::new(GitCommitTool::new(root.path().to_owned())),
            serde_json::json!({"message": "fixture"}),
        ),
        (
            Box::new(GitCheckoutTool::new(root.path().to_owned())),
            serde_json::json!({"branch": "fixture"}),
        ),
        (
            Box::new(GitPushTool::new(root.path().to_owned())),
            serde_json::json!({}),
        ),
        (
            Box::new(GitPullTool::new(root.path().to_owned())),
            serde_json::json!({}),
        ),
    ];
    for (tool, input) in cases {
        let result = tool.prepare_normalized_with_provenance(&input, ToolProvenance::Native);
        let Err(error) = result else {
            panic!("{} enabled production preparation", tool.name());
        };
        assert!(
            error.contains("disabled pending native acceptance of Job-supervised execution"),
            "{}: {error}",
            tool.name()
        );
    }
    assert_eq!(
        std::fs::read_dir(root.path())
            .expect("fixture entries")
            .count(),
        0
    );
}
