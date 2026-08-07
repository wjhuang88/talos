use std::fs;
use std::path::PathBuf;

fn source_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

#[test]
fn tool_facade_keeps_responsibility_modules_private() {
    let facade = fs::read_to_string(source_path("src/tool.rs")).expect("read tool facade");

    for module in [
        "agent_tool",
        "authorization",
        "protocol",
        "registry",
        "result_presentation",
    ] {
        assert!(
            facade.contains(&format!("mod {module};")),
            "facade must declare private module {module}"
        );
        assert!(
            !facade.contains(&format!("pub mod {module};")),
            "implementation module {module} must remain private"
        );
    }

    assert!(facade.contains("pub use self::{"));
    assert!(facade.contains("macro_rules! tool_parameters"));
    assert!(!facade.contains("pub struct "));
    assert!(!facade.contains("pub enum "));
    assert!(!facade.contains("pub trait "));
    assert!(!facade.contains("impl AgentTool"));
    assert!(!facade.contains("impl ToolRegistry"));
}

#[test]
fn tool_responsibilities_have_focused_source_owners() {
    let cases = [
        ("src/tool/agent_tool.rs", "pub trait AgentTool"),
        (
            "src/tool/authorization.rs",
            "pub struct ToolExecutionAuthorization",
        ),
        ("src/tool/protocol.rs", "pub enum ToolProtocol"),
        ("src/tool/registry.rs", "pub struct ToolRegistry"),
        (
            "src/tool/result_presentation.rs",
            "pub struct ToolPresentationPolicy",
        ),
        ("src/tool/tests.rs", "fn test_register_and_get_tool"),
    ];

    for (path, declaration) in cases {
        let source = fs::read_to_string(source_path(path)).expect("read responsibility source");
        assert!(
            source.contains(declaration),
            "{path} must own {declaration}"
        );
    }
}
