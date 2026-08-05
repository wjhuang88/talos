from pathlib import Path
import re

FILES = [
    "crates/talos-core/src/submission.rs",
    "crates/talos-provider/src/anthropic_request.rs",
    "crates/talos-provider/src/anthropic_stream.rs",
    "crates/talos-tui/src/app.rs",
    "crates/talos-tui/src/app_layout.rs",
    "crates/talos-tui/src/app_summary.rs",
    "crates/talos-tui/src/scrollback_status_git.rs",
    "crates/talos-tui/src/state_tests.rs",
    "crates/talos-tui/src/tool_display.rs",
    "crates/talos-cli/src/approval.rs",
    "crates/talos-cli/src/i169_bridge_integration_tests.rs",
    "crates/talos-cli/src/image_authorization.rs",
    "crates/talos-cli/src/init_wizard.rs",
    "crates/talos-cli/src/mode_interactive.rs",
    "crates/talos-cli/src/mode_print.rs",
    "crates/talos-cli/src/mode_runners_tests.rs",
    "crates/talos-cli/src/mode_runtime.rs",
    "crates/talos-cli/src/model_lifecycle.rs",
    "crates/talos-cli/src/models_browser.rs",
    "crates/talos-cli/src/permissions.rs",
    "crates/talos-cli/src/provider_discovery.rs",
    "crates/talos-cli/src/recent_models.rs",
    "crates/talos-cli/src/registry.rs",
    "crates/talos-cli/src/session_handlers.rs",
    "crates/talos-cli/src/session_transition.rs",
    "crates/talos-cli/src/storage.rs",
    "crates/talos-cli/src/todo_view.rs",
    "crates/talos-cli/src/tui_bridge.rs",
    "crates/talos-cli/src/tui_runtime_builder.rs",
]

for filename in FILES:
    path = Path(filename)
    text = path.read_text()

    # Preserve failure assertions while avoiding clippy::unwrap_used.
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
