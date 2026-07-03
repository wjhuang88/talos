use std::path::Path;

pub(crate) fn format_governance_summary(workspace: &Path) -> String {
    let mut out = String::new();
    out.push_str("[System] Talos Governance Status\n");
    out.push_str("[System] ========================\n\n");

    format_manifest(workspace, &mut out);
    format_board(workspace, &mut out);
    format_iterations(workspace, &mut out);
    format_backlog(workspace, &mut out);
    format_validation(workspace, &mut out);

    out
}

fn format_manifest(workspace: &Path, out: &mut String) {
    let path = workspace.join(".agent-governance").join("manifest.yaml");
    let Ok(content) = std::fs::read_to_string(&path) else {
        out.push_str("[System] Manifest: not found\n\n");
        return;
    };

    out.push_str("[System] Manifest\n");
    for line in content.lines() {
        let t = line.trim();
        for key in ["profile", "status", "last_audited_at"] {
            if let Some(val) = t.strip_prefix(&format!("{key}:")) {
                out.push_str(&format!(
                    "[System]   {key}: {}\n",
                    val.trim().trim_matches('"')
                ));
            }
        }
    }
    out.push('\n');
}

fn format_board(workspace: &Path, out: &mut String) {
    let path = workspace.join("docs").join("BOARD.md");
    let Ok(content) = std::fs::read_to_string(&path) else {
        out.push_str("[System] Board: not found\n\n");
        return;
    };

    out.push_str("[System] Board Disposition\n");
    for heading in ["Now", "Blocked / Paused", "Next"] {
        let items = parse_board_section(&content, heading);
        out.push_str(&format!("[System]   {heading}: {} item(s)\n", items.len()));
        for item in items {
            out.push_str(&format!("[System]     - {} [{}]\n", item.0, item.1));
        }
    }
    out.push('\n');
}

fn parse_board_section(content: &str, heading: &str) -> Vec<(String, String)> {
    let target = format!("## {heading}");
    let mut in_section = false;
    let mut items = Vec::new();

    for line in content.lines() {
        if line.starts_with("## ") {
            in_section = line.trim() == target;
            continue;
        }
        if !in_section
            || !line.starts_with("| ")
            || line.starts_with("|---")
            || line.starts_with("| Item")
        {
            continue;
        }
        let cols: Vec<&str> = line.split('|').collect();
        if cols.len() < 5 {
            continue;
        }
        let item = clean_cell(cols[1]);
        let state = clean_cell(cols[2]);
        if !item.is_empty() && !item.starts_with("_(no active") {
            items.push((item, state));
        }
    }

    items
}

fn clean_cell(cell: &str) -> String {
    cell.trim()
        .trim_matches('`')
        .replace('*', "")
        .trim()
        .to_string()
}

fn format_iterations(workspace: &Path, out: &mut String) {
    let path = workspace.join("docs").join("iterations").join("README.md");
    let Ok(content) = std::fs::read_to_string(&path) else {
        out.push_str("[System] Iterations: not found\n\n");
        return;
    };

    let open = parse_open_iterations(&content);
    out.push_str("[System] Open Iterations\n");
    if open.is_empty() {
        out.push_str("[System]   (all iterations complete)\n");
    }
    for iter in &open {
        out.push_str(&format!("[System]   {} {} — {}\n", iter.0, iter.1, iter.2));
    }
    out.push('\n');
}

fn parse_open_iterations(content: &str) -> Vec<(String, String, String)> {
    let mut in_current = false;
    let mut items = Vec::new();

    for line in content.lines() {
        if line.starts_with("## ") {
            in_current = line.starts_with("## Current Iterations");
            continue;
        }
        if !in_current
            || !line.starts_with("| ")
            || line.starts_with("|---")
            || line.starts_with("| ID")
        {
            continue;
        }
        let cols: Vec<&str> = line.split('|').collect();
        if cols.len() < 5 {
            continue;
        }
        let id = clean_cell(cols[1]);
        let codename = clean_cell(cols[2]);
        let state = clean_cell(cols[3]).to_lowercase();
        if is_open_state(&state) {
            items.push((id, codename, state));
        }
    }

    items
}

fn is_open_state(state: &str) -> bool {
    if state.contains("complete") || state.contains("superseded") {
        return false;
    }
    [
        "planned",
        "active",
        "paused",
        "review",
        "blocked",
        "refinement",
        "tracking",
    ]
    .iter()
    .any(|s| state.contains(s))
}

fn format_backlog(workspace: &Path, out: &mut String) {
    let path = workspace
        .join("docs")
        .join("backlog")
        .join("PRODUCT-BACKLOG.md");
    let Ok(content) = std::fs::read_to_string(&path) else {
        out.push_str("[System] Backlog: not found\n\n");
        return;
    };

    let active = count_backlog_active_items(&content);
    out.push_str("[System] Backlog\n");
    out.push_str(&format!("[System]   Active items: {}\n", active.len()));
    let in_progress: Vec<&str> = active
        .iter()
        .filter(|(_, s)| {
            let l = s.to_lowercase();
            l.contains("progress") || l.contains("active") || l.contains("review")
        })
        .map(|(id, _)| id.as_str())
        .collect();
    if !in_progress.is_empty() {
        out.push_str(&format!(
            "[System]   In progress: {}\n",
            in_progress.join(", ")
        ));
    }
    out.push('\n');
}

fn count_backlog_active_items(content: &str) -> Vec<(String, String)> {
    let mut in_active = false;
    let mut items = Vec::new();

    for line in content.lines() {
        if line.starts_with("## ") {
            in_active = line.starts_with("## Active Items");
            continue;
        }
        if !in_active
            || !line.starts_with("| ")
            || line.starts_with("|---")
            || line.starts_with("| ID")
        {
            continue;
        }
        let cols: Vec<&str> = line.split('|').collect();
        if cols.len() < 5 {
            continue;
        }
        let id = clean_cell(cols[1]);
        let status = clean_cell(cols[3]);
        if !id.is_empty() && !id.starts_with("_(no") {
            items.push((id, status));
        }
    }

    items
}

fn format_validation(workspace: &Path, out: &mut String) {
    let report = collect_governance_validation(workspace);
    out.push_str("[System] Validation\n");
    if report.errors == 0 {
        out.push_str(&format!(
            "[System]   Status: PASS ({} warning(s))\n",
            report.warnings
        ));
    } else {
        out.push_str(&format!(
            "[System]   Status: FAIL ({} error(s), {} warning(s))\n",
            report.errors, report.warnings
        ));
    }
    for finding in report.findings.iter().take(12) {
        out.push_str(&format!("[System]   {finding}\n"));
    }
    if report.findings.len() > 12 {
        out.push_str(&format!(
            "[System]   ... {} more finding(s)\n",
            report.findings.len() - 12
        ));
    }
    out.push('\n');
}

/// Programmatic, read-only governance validation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernanceValidationReport {
    /// Number of blocking validation errors.
    pub errors: usize,
    /// Number of non-blocking validation warnings.
    pub warnings: usize,
    /// Human-readable findings prefixed with `ERROR:`, `WARNING:`, or `OK:`.
    pub findings: Vec<String>,
}

impl GovernanceValidationReport {
    fn new() -> Self {
        Self {
            errors: 0,
            warnings: 0,
            findings: Vec::new(),
        }
    }

    fn ok(&mut self, message: impl Into<String>) {
        self.findings.push(format!("OK: {}", message.into()));
    }

    fn warn(&mut self, message: impl Into<String>) {
        self.warnings += 1;
        self.findings.push(format!("WARNING: {}", message.into()));
    }

    fn error(&mut self, message: impl Into<String>) {
        self.errors += 1;
        self.findings.push(format!("ERROR: {}", message.into()));
    }
}

/// Validate Talos project governance using Rust-only, read-only checks.
#[must_use]
pub fn collect_governance_validation(workspace: &Path) -> GovernanceValidationReport {
    let mut report = GovernanceValidationReport::new();
    let manifest_path = workspace.join(".agent-governance").join("manifest.yaml");
    let Ok(manifest) = std::fs::read_to_string(&manifest_path) else {
        report.error("missing .agent-governance/manifest.yaml");
        return report;
    };

    let profile = manifest_value(&manifest, "profile");
    let status = manifest_value(&manifest, "status");
    let product_like = matches!(profile.as_deref(), Some("product" | "high-risk"));

    report.ok(".agent-governance/manifest.yaml found");
    validate_entrypoints(workspace, &manifest, &mut report);
    validate_product_profile(
        workspace,
        &manifest,
        product_like,
        status.as_deref(),
        &mut report,
    );
    validate_iteration_records(workspace, &manifest, &mut report);
    validate_board(workspace, &mut report);
    validate_capability_files(workspace, &manifest, &mut report);
    validate_policy_text(workspace, &manifest, product_like, &mut report);

    if matches!(status.as_deref(), Some("degraded" | "adopting")) {
        report.warn(format!(
            "manifest status is '{}': verify declared capabilities before relying on governance state",
            status.unwrap_or_default()
        ));
    }

    report
}

fn manifest_value(content: &str, key: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let trimmed = strip_yaml_comment(line.trim());
        trimmed
            .strip_prefix(&format!("{key}:"))
            .map(clean_yaml_value)
            .filter(|value| !value.is_empty())
    })
}

fn section_value(content: &str, section: &str, key: &str) -> Option<String> {
    section_pairs(content, section)
        .into_iter()
        .find_map(|(name, value)| (name == key).then_some(value))
}

fn section_pairs(content: &str, section: &str) -> Vec<(String, String)> {
    let mut current = "";
    let mut pairs = Vec::new();
    for line in content.lines() {
        let trimmed = strip_yaml_comment(line.trim_end());
        if trimmed.is_empty() {
            continue;
        }
        if !line.starts_with(' ') && trimmed.ends_with(':') {
            current = trimmed.trim_end_matches(':');
            continue;
        }
        if current == section && line.starts_with("  ") {
            let t = strip_yaml_comment(line.trim());
            if let Some((key, value)) = t.split_once(':') {
                let value = clean_yaml_value(value);
                if !key.trim().is_empty() && !value.is_empty() {
                    pairs.push((key.trim().to_string(), value));
                }
            }
        }
    }
    pairs
}

fn capability_value(content: &str, capability: &str) -> Option<String> {
    section_value(content, "capabilities", capability)
}

fn strip_yaml_comment(line: &str) -> &str {
    line.split_once('#').map_or(line, |(head, _)| head).trim()
}

fn clean_yaml_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_string()
}

fn validate_entrypoints(workspace: &Path, manifest: &str, report: &mut GovernanceValidationReport) {
    for (name, target) in section_pairs(manifest, "entrypoints") {
        if !workspace.join(&target).exists() {
            report.error(format!(
                "declared entrypoint does not exist: {name} -> {target}"
            ));
        }
    }
}

fn validate_product_profile(
    workspace: &Path,
    manifest: &str,
    product_like: bool,
    status: Option<&str>,
    report: &mut GovernanceValidationReport,
) {
    if !product_like {
        return;
    }

    for capability in [
        "task_router",
        "evolution_feedback",
        "testing_policy",
        "git_workflow",
        "requirement_intake",
        "iteration_workflow",
        "change_control",
    ] {
        if capability_value(manifest, capability).as_deref() == Some("not_applicable") {
            let profile = manifest_value(manifest, "profile").unwrap_or_default();
            report.error(format!(
                "{profile} profile cannot mark {capability} as not_applicable"
            ));
        }
    }

    check_status_sensitive_file(
        workspace,
        "docs/README.md",
        "product governance is missing documentation map: docs/README.md",
        status,
        report,
    );
    check_status_sensitive_file(
        workspace,
        "docs/sop/DOC-CHECK.md",
        "multi-layer product governance is missing docs/sop/DOC-CHECK.md",
        status,
        report,
    );
}

fn check_status_sensitive_file(
    workspace: &Path,
    relative_path: &str,
    message: &str,
    status: Option<&str>,
    report: &mut GovernanceValidationReport,
) {
    if workspace.join(relative_path).is_file() {
        return;
    }
    if status == Some("conformant") {
        report.error(message);
    } else {
        report.warn(message);
    }
}

fn validate_iteration_records(
    workspace: &Path,
    manifest: &str,
    report: &mut GovernanceValidationReport,
) {
    let records = iteration_record_paths(workspace);
    if !records.is_empty()
        && capability_value(manifest, "iteration_workflow").as_deref() == Some("not_applicable")
    {
        report.error("iteration records exist but iteration_workflow is marked not_applicable");
    }

    for record in records {
        let Ok(content) = std::fs::read_to_string(&record) else {
            continue;
        };
        if claims_completion(&content) && !shows_evidence(&content) {
            let relative = record
                .strip_prefix(workspace)
                .ok()
                .and_then(|p| p.to_str())
                .unwrap_or_else(|| record.to_str().unwrap_or("<invalid path>"));
            report.warn(format!(
                "iteration claims completion but records no validation evidence: {}",
                relative.trim_start_matches('/')
            ));
        }
    }
}

fn iteration_record_paths(workspace: &Path) -> Vec<std::path::PathBuf> {
    let dir = workspace.join("docs").join("iterations");
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().and_then(|ext| ext.to_str()) == Some("md")
                && !matches!(
                    path.file_name().and_then(|name| name.to_str()),
                    Some("README.md" | "readme.md")
                )
        })
        .collect()
}

fn claims_completion(content: &str) -> bool {
    let lower = content.to_ascii_lowercase();
    ["complete", "completed", "done", "shipped", "delivered"]
        .iter()
        .any(|word| lower.contains(word))
}

fn shows_evidence(content: &str) -> bool {
    let lower = content.to_ascii_lowercase();
    content.contains("```")
        || [
            "test",
            "tests",
            "tested",
            "testing",
            "cargo",
            "passed",
            "passing",
            "verified",
            "verify",
            "evidence",
            "exit 0",
            "coverage",
            "benchmark",
            "smoke",
        ]
        .iter()
        .any(|word| lower.contains(word))
}

fn validate_board(workspace: &Path, report: &mut GovernanceValidationReport) {
    let path = workspace.join("docs").join("BOARD.md");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return;
    };
    let lower = content.to_ascii_lowercase();
    if !(lower.contains("derived operating view") || lower.contains("derived-operating-view")) {
        report
            .warn("docs/BOARD.md exists but is not explicitly marked as a derived operating view");
    }
    if !content.contains("Owner Doc") {
        report.warn(
            "docs/BOARD.md exists but does not include an Owner Doc column or equivalent label",
        );
    }
    if !content.contains("Gate") {
        report.warn("docs/BOARD.md exists but does not include a Gate column or equivalent label");
    }
}

fn validate_capability_files(
    workspace: &Path,
    manifest: &str,
    report: &mut GovernanceValidationReport,
) {
    for (capability, required) in [
        ("task_router", &["AGENTS.md"][..]),
        (
            "evolution_feedback",
            &["EVOLUTION.md", "docs/sop/EVOLUTION-FEEDBACK.md"][..],
        ),
        ("long_running_task", &["docs/sop/LONG-RUNNING-TASK.md"][..]),
        ("testing_policy", &["docs/sop/TESTING.md"][..]),
        ("git_workflow", &["docs/sop/GIT-WORKFLOW.md"][..]),
        (
            "requirement_intake",
            &["docs/sop/REQUIREMENT-INTAKE.md"][..],
        ),
        (
            "iteration_workflow",
            &[
                "docs/sop/START-ITERATION.md",
                "docs/sop/ITERATION-WORKFLOW.md",
            ][..],
        ),
        ("change_control", &["docs/sop/CHANGE-CONTROL.md"][..]),
        ("decision_records", &["docs/decisions/README.md"][..]),
        ("release_workflow", &["docs/sop/RELEASE.md"][..]),
    ] {
        if capability_value(manifest, capability).as_deref() == Some("conformant") {
            for relative in required {
                if !workspace.join(relative).exists() {
                    report.error(format!(
                        "{capability} is conformant but required file is missing: {relative}"
                    ));
                }
                if !matches!(capability, "task_router" | "evolution_feedback")
                    && !file_contains(workspace, "AGENTS.md", relative)
                {
                    report.error(format!(
                        "conformant recurring workflow is not routed from AGENTS.md: {capability} -> {relative}"
                    ));
                }
            }
        }
    }
}

fn validate_policy_text(
    workspace: &Path,
    manifest: &str,
    product_like: bool,
    report: &mut GovernanceValidationReport,
) {
    if capability_value(manifest, "evolution_feedback").as_deref() == Some("conformant")
        && !file_contains(workspace, "AGENTS.md", "docs/sop/EVOLUTION-FEEDBACK.md")
    {
        report.error("conformant evolution_feedback is not routed from AGENTS.md: docs/sop/EVOLUTION-FEEDBACK.md");
    }

    if capability_value(manifest, "iteration_workflow").as_deref() == Some("conformant") {
        for (file, text, message) in [
            (
                "docs/iterations/TEMPLATE.md",
                "Published plan date",
                "iteration template is missing published baseline metadata: Published plan date",
            ),
            (
                "AGENTS.md",
                "published baseline",
                "AGENTS.md does not expose the published iteration baseline rule",
            ),
            (
                "docs/sop/START-ITERATION.md",
                "Inventory Existing Iterations",
                "START-ITERATION does not require non-terminal iteration inventory",
            ),
            (
                "docs/sop/START-ITERATION.md",
                "runnable, testable deliverable",
                "START-ITERATION does not require a runnable, testable deliverable",
            ),
            (
                "docs/sop/CHANGE-CONTROL.md",
                "Never overwrite a published iteration baseline",
                "CHANGE-CONTROL does not preserve published iteration baselines",
            ),
        ] {
            if !file_contains(workspace, file, text) {
                report.error(message);
            }
        }
    }

    if capability_value(manifest, "requirement_intake").as_deref() == Some("conformant") {
        for text in [
            "Given/When/Then",
            "Required Reads",
            "Decision links",
            "user-facing documentation",
        ] {
            if !file_contains(workspace, "docs/sop/REQUIREMENT-INTAKE.md", text) {
                report.error(format!(
                    "REQUIREMENT-INTAKE is missing current ready-story rule: {text}"
                ));
            }
        }
    }

    if capability_value(manifest, "long_running_task").as_deref() == Some("conformant") {
        for text in [
            "Startup Contract",
            "Consolidated Confirmation",
            "Recovery or resume instruction",
            "Completion Gate",
        ] {
            if !file_contains(workspace, "docs/sop/LONG-RUNNING-TASK.md", text) {
                report.error(format!(
                    "LONG-RUNNING-TASK is missing required contract section: {text}"
                ));
            }
        }
        if !file_contains(workspace, "AGENTS.md", "docs/sop/LONG-RUNNING-TASK.md") {
            report.error("conformant long-running task workflow is not routed from AGENTS.md");
        }
    }

    if product_like && capability_value(manifest, "task_router").as_deref() == Some("conformant") {
        for section in [
            "Hard Constraints",
            "Coding Behavior",
            "Git Rules",
            "Task Router",
            "Session End Checklist",
        ] {
            if !file_contains(workspace, "AGENTS.md", section) {
                report.error(format!("AGENTS.md is missing required section: {section}"));
            }
        }
        if !file_contains(workspace, "AGENTS.md", "[model:<model-name>]") {
            report.error("AGENTS.md Git Rules must include the Agent commit model tag format");
        }
    }
}

fn file_contains(workspace: &Path, relative_path: &str, text: &str) -> bool {
    std::fs::read_to_string(workspace.join(relative_path))
        .map(|content| content.contains(text))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_workspace_does_not_panic() {
        let dir = tempfile::tempdir().unwrap();
        let out = format_governance_summary(dir.path());
        assert!(out.contains("not found"));
    }

    #[test]
    fn board_section_parsing() {
        let board = "# Board\n\n## Now\n\n| Item | State | Owner Doc | Gate |\n|---|---|---|---|\n| T27 Governance | Active | [x](x.md) | Gate |\n\n## Next\n\n| Item | State | Owner Doc | Gate |\n|---|---|---|---|\n| T38 Rehearsal | Planned | [x](x.md) | Evidence |\n";
        assert_eq!(parse_board_section(board, "Now").len(), 1);
        assert_eq!(parse_board_section(board, "Next").len(), 1);
        assert_eq!(parse_board_section(board, "Blocked / Paused").len(), 0);
    }

    #[test]
    fn open_iterations_filter() {
        let content = "## Current Iterations\n\n| ID | Codename | State | Verified |\n|---|---|---|---|\n| I018 | Obs | Planned | no |\n| I009 | Ext | **Complete** | yes |\n";
        let open = parse_open_iterations(content);
        assert_eq!(open.len(), 1);
        assert_eq!(open[0].0, "I018");
    }

    #[test]
    fn full_summary_with_files() {
        let dir = tempfile::tempdir().unwrap();
        let gov_dir = dir.path().join(".agent-governance");
        std::fs::create_dir_all(&gov_dir).unwrap();
        std::fs::write(
            gov_dir.join("manifest.yaml"),
            "profile: \"high-risk\"\nstatus: \"conformant\"\n",
        )
        .unwrap();

        let docs_dir = dir.path().join("docs");
        std::fs::create_dir_all(docs_dir.join("iterations")).unwrap();
        std::fs::write(
            docs_dir.join("BOARD.md"),
            "# Board\n\n## Now\n\n| Item | State | Owner Doc | Gate |\n|---|---|---|---|\n| I047 Test | Active | [x](x.md) | Gate |\n",
        )
        .unwrap();
        std::fs::write(
            docs_dir.join("iterations").join("README.md"),
            "## Current Iterations\n\n| ID | Codename | State | Verified |\n|---|---|---|---|\n| I080 | Frontline | Planned | no |\n",
        )
        .unwrap();

        let backlog_dir = docs_dir.join("backlog");
        std::fs::create_dir_all(&backlog_dir).unwrap();
        std::fs::write(
            backlog_dir.join("PRODUCT-BACKLOG.md"),
            "## Active Items\n\n| ID | Title | Status | Priority | Decision Context | Required Reads |\n|---|---|---|---|---|---|\n| TUI-021 | Composer Nav | In Progress | P3 | ctx | reads |\n| CONF-001 | Config | Planned | P2 | ctx | reads |\n",
        )
        .unwrap();

        let out = format_governance_summary(dir.path());
        assert!(out.contains("conformant"));
        assert!(out.contains("I047 Test"));
        assert!(out.contains("I080"));
        assert!(out.contains("Backlog"));
        assert!(out.contains("Active items: 2"));
        assert!(out.contains("TUI-021"));
    }

    #[test]
    fn backlog_parsing_counts_active_items() {
        let content = "## Active Items\n\n| ID | Title | Status | Priority | Decision Context | Required Reads |\n|---|---|---|---|---|---|\n| REL-002 | Release | Planned | P1 | ctx | reads |\n| GOV-003 | Governance | In Progress | P2 | ctx | reads |\n";
        let items = count_backlog_active_items(content);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].0, "REL-002");
        assert_eq!(items[1].0, "GOV-003");
    }

    #[test]
    fn validation_section_reports_missing_manifest() {
        let dir = tempfile::tempdir().unwrap();
        let mut out = String::new();
        format_validation(dir.path(), &mut out);
        assert!(out.contains("Status: FAIL"));
        assert!(out.contains("missing .agent-governance/manifest.yaml"));
    }

    #[test]
    fn validation_section_is_programmatic_without_script() {
        let dir = tempfile::tempdir().unwrap();
        let gov_dir = dir.path().join(".agent-governance");
        std::fs::create_dir_all(&gov_dir).unwrap();
        std::fs::write(
            gov_dir.join("manifest.yaml"),
            "profile: \"small\"\nstatus: \"conformant\"\n",
        )
        .unwrap();

        let mut out = String::new();
        format_validation(dir.path(), &mut out);

        assert!(out.contains("Status: PASS"));
        assert!(out.contains("OK: .agent-governance/manifest.yaml found"));
    }

    #[test]
    fn validation_detects_missing_conformant_capability_file() {
        let dir = tempfile::tempdir().unwrap();
        let gov_dir = dir.path().join(".agent-governance");
        std::fs::create_dir_all(&gov_dir).unwrap();
        std::fs::write(
            gov_dir.join("manifest.yaml"),
            "profile: \"small\"\nstatus: \"conformant\"\ncapabilities:\n  testing_policy: conformant\n",
        )
        .unwrap();

        let report = collect_governance_validation(dir.path());

        assert!(report.errors >= 1);
        assert!(report.findings.iter().any(|finding| finding.contains(
            "testing_policy is conformant but required file is missing: docs/sop/TESTING.md"
        )));
    }
}
