//! Workspace eligibility shared by product tool composition roots.

use std::path::Path;

/// Whether the workspace has enough governance structure for agile discovery.
/// This enables discovery only, not automatic invocation or mutation authority.
pub fn agile_summary_available(workspace: &Path) -> bool {
    workspace.join(".agent-governance/manifest.yaml").is_file()
        || [
            "docs/BOARD.md",
            "docs/iterations/README.md",
            "docs/backlog/PRODUCT-BACKLOG.md",
        ]
        .iter()
        .filter(|marker| workspace.join(marker).is_file())
        .count()
            >= 2
}

/// Whether existing project detection supports exposing validation planning.
/// Detection does not promise a host adapter for every recognized ecosystem.
pub fn project_validation_available(workspace: &Path) -> bool {
    !crate::detect_project_types(workspace).is_empty() || agile_summary_available(workspace)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marker(root: &Path, relative: &str) {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("directory");
        std::fs::write(path, "# Planned\n").expect("marker");
    }

    #[test]
    fn language_projects_do_not_need_governance_to_discover_validation() {
        for language_marker in [
            "Cargo.toml",
            "package.json",
            "pyproject.toml",
            "go.mod",
            "pom.xml",
        ] {
            let root = tempfile::tempdir().expect("workspace");
            marker(root.path(), language_marker);
            assert!(project_validation_available(root.path()));
            assert!(!agile_summary_available(root.path()));
        }
    }

    #[test]
    fn every_governance_pair_qualifies_without_an_active_iteration() {
        let markers = [
            "docs/BOARD.md",
            "docs/iterations/README.md",
            "docs/backlog/PRODUCT-BACKLOG.md",
        ];
        for first in 0..markers.len() {
            let root = tempfile::tempdir().expect("workspace");
            marker(root.path(), markers[first]);
            assert!(!agile_summary_available(root.path()));
            for second in first + 1..markers.len() {
                let pair = tempfile::tempdir().expect("workspace");
                marker(pair.path(), markers[first]);
                marker(pair.path(), markers[second]);
                assert!(agile_summary_available(pair.path()));
                assert!(project_validation_available(pair.path()));
            }
        }
    }

    #[test]
    fn manifest_alone_qualifies_but_empty_workspace_and_fake_directory_do_not() {
        let root = tempfile::tempdir().expect("workspace");
        assert!(!agile_summary_available(root.path()));
        assert!(!project_validation_available(root.path()));
        std::fs::create_dir_all(root.path().join("docs/BOARD.md")).expect("fake marker");
        assert!(!agile_summary_available(root.path()));
        marker(root.path(), ".agent-governance/manifest.yaml");
        assert!(agile_summary_available(root.path()));
        assert!(project_validation_available(root.path()));
    }
}
