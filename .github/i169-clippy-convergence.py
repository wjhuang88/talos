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


for path in sorted(INLINE_TEST_FILES | TEST_FILES):
    clarify_test_failures(path)

# Known needless-borrow diagnostics that are not dependent on formatting.
path = Path("crates/talos-provider/src/anthropic_request.rs")
text = path.read_text().replace("&png_header", "png_header")
path.write_text(text)

path = Path("crates/talos-cli/src/registry.rs")
text = path.read_text().replace(
    "&[0x89, 0x50, 0x4E, 0x47]",
    "[0x89, 0x50, 0x4E, 0x47]",
)
path.write_text(text)
