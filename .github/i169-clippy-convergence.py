from pathlib import Path
import re

# Inline test modules that live in production source files and have already been
# identified by strict all-targets Clippy. Integration and extracted test files
# are discovered separately below, so production code is not globally rewritten.
INLINE_TEST_FILES = {
    Path("crates/talos-core/src/submission.rs"),
    Path("crates/talos-config/src/endpoint.rs"),
    Path("crates/talos-agent/src/scheduler.rs"),
    Path("crates/talos-provider/src/anthropic_request.rs"),
    Path("crates/talos-provider/src/anthropic_stream.rs"),
    Path("crates/talos-tui/src/app.rs"),
    Path("crates/talos-tui/src/app_layout.rs"),
    Path("crates/talos-tui/src/app_summary.rs"),
    Path("crates/talos-tui/src/scrollback_status_git.rs"),
    Path("crates/talos-tui/src/tool_display.rs"),
    Path("crates/talos-cli/src/approval.rs"),
    Path("crates/talos-cli/src/image_authorization.rs"),
    Path("crates/talos-cli/src/init_wizard.rs"),
    Path("crates/talos-cli/src/mode_interactive.rs"),
    Path("crates/talos-cli/src/mode_print.rs"),
    Path("crates/talos-cli/src/mode_runtime.rs"),
    Path("crates/talos-cli/src/model_lifecycle.rs"),
    Path("crates/talos-cli/src/models_browser.rs"),
    Path("crates/talos-cli/src/permissions.rs"),
    Path("crates/talos-cli/src/provider_discovery.rs"),
    Path("crates/talos-cli/src/recent_models.rs"),
    Path("crates/talos-cli/src/registry.rs"),
    Path("crates/talos-cli/src/session_handlers.rs"),
    Path("crates/talos-cli/src/session_transition.rs"),
    Path("crates/talos-cli/src/storage.rs"),
    Path("crates/talos-cli/src/todo_view.rs"),
    Path("crates/talos-cli/src/tui_bridge.rs"),
    Path("crates/talos-cli/src/tui_runtime_builder.rs"),
}

TEST_FILES = {
    path
    for path in Path("crates").rglob("*.rs")
    if "tests" in path.parts
    or path.name == "tests.rs"
    or path.name.endswith("_tests.rs")
}


def clarify_test_failures(path: Path) -> None:
    text = path.read_text()
    text = re.sub(
        r"\.err\(\)\s*\.expect\(([^\n]+?)\)",
        r".expect_err(\1)",
        text,
    )
    text = text.replace(
        ".unwrap_err()",
        '.expect_err("operation should fail")',
    )
    text = text.replace(
        ".unwrap()",
        '.expect("operation should succeed")',
    )
    path.write_text(text)


def rewrite_initial_default_assignments(path: Path, test_suffix_only: bool) -> None:
    text = path.read_text()
    prefix = ""
    target = text
    if test_suffix_only:
        marker = text.find("#[cfg(test)]")
        if marker < 0:
            return
        prefix, target = text[:marker], text[marker:]

    lines = target.splitlines(keepends=True)
    output: list[str] = []
    i = 0
    declaration = re.compile(
        r"^(?P<indent>\s*)let mut (?P<var>[A-Za-z_][A-Za-z0-9_]*) = "
        r"(?P<type>[A-Za-z_][A-Za-z0-9_:]*)::default\(\);\s*$"
    )
    while i < len(lines):
        line_without_newline = lines[i].rstrip("\r\n")
        match = declaration.match(line_without_newline)
        if match is None:
            output.append(lines[i])
            i += 1
            continue

        indent = match.group("indent")
        var = match.group("var")
        type_name = match.group("type")
        assignments: list[tuple[str, str]] = []
        j = i + 1
        assignment = re.compile(
            rf"^{re.escape(indent)}{re.escape(var)}\."
            r"(?P<field>[A-Za-z_][A-Za-z0-9_]*) = (?P<value>.+);\s*$"
        )
        while j < len(lines):
            candidate = lines[j].rstrip("\r\n")
            assigned = assignment.match(candidate)
            if assigned is None:
                break
            assignments.append((assigned.group("field"), assigned.group("value")))
            j += 1

        if not assignments:
            output.append(lines[i])
            i += 1
            continue

        output.append(f"{indent}let mut {var} = {type_name} {{\n")
        for field, value in assignments:
            output.append(f"{indent}    {field}: {value},\n")
        output.append(f"{indent}    ..Default::default()\n")
        output.append(f"{indent}}};\n")
        i = j

    path.write_text(prefix + "".join(output))


def remove_first_builder_argument(text: str) -> str:
    needle = "TuiRuntimeBuilder::new("
    position = 0
    while True:
        start = text.find(needle, position)
        if start < 0:
            return text
        open_paren = start + len(needle) - 1
        i = open_paren + 1
        depth = 0
        in_string = False
        escaped = False
        while i < len(text):
            char = text[i]
            if in_string:
                if escaped:
                    escaped = False
                elif char == "\\":
                    escaped = True
                elif char == '"':
                    in_string = False
            else:
                if char == '"':
                    in_string = True
                elif char in "([{":
                    depth += 1
                elif char in ")]}":
                    if depth > 0:
                        depth -= 1
                elif char == "," and depth == 0:
                    text = text[: open_paren + 1] + text[i + 1 :]
                    position = open_paren + 1
                    break
            i += 1
        else:
            raise RuntimeError("unterminated TuiRuntimeBuilder::new call")


for path in sorted(INLINE_TEST_FILES | TEST_FILES):
    clarify_test_failures(path)

for path in sorted(TEST_FILES):
    rewrite_initial_default_assignments(path, test_suffix_only=False)
for path in sorted(INLINE_TEST_FILES):
    rewrite_initial_default_assignments(path, test_suffix_only=True)

# HOME is process-global. Use one async-aware mutex so asynchronous tests remain
# serialized without carrying a std::sync::MutexGuard across await points.
path = Path("crates/talos-cli/src/test_support.rs")
text = path.read_text().replace(
    "pub(crate) static HOME_ENV_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());",
    "pub(crate) static HOME_ENV_MUTEX: tokio::sync::Mutex<()> = "
    "tokio::sync::Mutex::const_new(());",
)
path.write_text(text)
for filename in [
    "crates/talos-cli/src/init_wizard.rs",
    "crates/talos-cli/src/mode_runners_tests.rs",
]:
    path = Path(filename)
    text = re.sub(
        r"HOME_ENV_MUTEX\.lock\(\)\.(?:unwrap\(\)|expect\([^\n]*?\))",
        "HOME_ENV_MUTEX.lock().await",
        path.read_text(),
    )
    path.write_text(text)

# Keep synchronous test guards in lexical scopes that end before runtime awaits.
path = Path("crates/talos-runtime/src/lib.rs")
text = path.read_text()
text = text.replace(
    '''        let records = approval_records.lock().expect("records lock is available");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].tool_name, "record_write");
        assert_eq!(
            records[0].arguments,
            serde_json::json!({"message": "approved"})
        );
        assert_eq!(records[0].summary_fields, vec!["message"]);

        runtime.shutdown().await.expect("shutdown succeeds");''',
    '''        {
            let records = approval_records.lock().expect("records lock is available");
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].tool_name, "record_write");
            assert_eq!(
                records[0].arguments,
                serde_json::json!({"message": "approved"})
            );
            assert_eq!(records[0].summary_fields, vec!["message"]);
        }

        runtime.shutdown().await.expect("shutdown succeeds");''',
)
text = text.replace(
    '''        let records = approval_records.lock().expect("records lock");
        assert_eq!(records.len(), 1);
        assert_eq!(
            records[0].arguments,
            serde_json::json!({"path": "src/lib.rs"})
        );
        drop(records);
        runtime.shutdown().await.expect("shutdown succeeds");''',
    '''        {
            let records = approval_records.lock().expect("records lock");
            assert_eq!(records.len(), 1);
            assert_eq!(
                records[0].arguments,
                serde_json::json!({"path": "src/lib.rs"})
            );
        }
        runtime.shutdown().await.expect("shutdown succeeds");''',
)
path.write_text(text)

# Remove the superseded non-activation quiesce path. I169 uses the atomic
# activation-specific boundary exclusively.
path = Path("crates/talos-cli/src/session_transition.rs")
text = re.sub(
    r"\n    pub async fn quiesce_same_session\(.*?\n    }\n(?=\n    pub fn prepare)",
    "",
    path.read_text(),
    flags=re.DOTALL,
)
path.write_text(text)

# Remove construction state that is no longer consumed after the runtime builder
# became the single authoritative reconstruction boundary.
path = Path("crates/talos-cli/src/tui_runtime_builder.rs")
text = path.read_text()
text = text.replace(
    "    ui_tx: mpsc::UnboundedSender<talos_conversation::UiOutput>,\n",
    "",
)
text = text.replace(
    "        ui_tx: mpsc::UnboundedSender<talos_conversation::UiOutput>,\n",
    "",
)
text = text.replace("            ui_tx,\n", "")
text = text.replace("    pub runtime_config: Config,\n", "")
text = text.replace("            runtime_config: self.runtime_config,\n", "")
text = re.sub(
    r"\n    #\[must_use\]\n    pub\(crate\) fn approval_handler\(&self\) -> Arc<TuiApprovalHandler> \{\n"
    r"        self\.approval_handler\.clone\(\)\n    \}\n",
    "",
    text,
)
path.write_text(text)
for path in Path("crates").rglob("*.rs"):
    text = path.read_text()
    if "TuiRuntimeBuilder::new(" in text:
        path.write_text(remove_first_builder_argument(text))

# Express the ambiguity search as iteration rather than index-only traversal.
path = Path("crates/talos-cli/src/todo_view.rs")
text = path.read_text().replace(
    '''                for j in (i + 1)..items.len() {
                    let prefix_j: String = items[j].id.to_string().chars().take(len).collect();''',
    '''                for item in items.iter().skip(i + 1) {
                    let prefix_j: String = item.id.to_string().chars().take(len).collect();''',
)
path.write_text(text)

# Known diagnostics that require imports or exact type changes beyond clippy --fix.
path = Path("crates/talos-tools/tests/document_boundaries.rs")
text = path.read_text()
text = text.replace("use std::path::PathBuf;", "use std::path::{Path, PathBuf};")
text = text.replace("fn cleanup(path: &PathBuf)", "fn cleanup(path: &Path)")
text = text.replace("fn run_extract(path: &PathBuf)", "fn run_extract(path: &Path)")
path.write_text(text)

path = Path("crates/talos-provider/src/anthropic_request.rs")
text = path.read_text().replace("&png_header", "png_header")
path.write_text(text)

path = Path("crates/talos-cli/src/registry.rs")
text = path.read_text().replace(
    "&[0x89, 0x50, 0x4E, 0x47]",
    "[0x89, 0x50, 0x4E, 0x47]",
)
path.write_text(text)
