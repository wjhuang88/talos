from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if old in text:
        count = text.count(old)
        if count != 1:
            raise SystemExit(f"{label}: expected one old form, found {count}")
        return text.replace(old, new, 1)
    if new not in text:
        raise SystemExit(f"{label}: neither old nor new form found")
    return text


path = Path("crates/talos-tools/src/bash_tool.rs")
text = path.read_text(encoding="utf-8")

old = '''fn is_simple_shell_token(token: &str) -> bool {
    !token.is_empty()
        && !token.contains('*')
        && !token.contains('?')
        && !token.contains('[')
        && !token.contains(']')
        && !token.contains('{')
        && !token.contains('}')
        && !token.contains('\\'')
        && !token.contains('"')
        && !token.contains('\\\\')
}
'''
new = '''fn is_simple_shell_token(token: &str) -> bool {
    !token.is_empty()
        && !token.contains('*')
        && !token.contains('?')
        && !token.contains('[')
        && !token.contains(']')
        && !token.contains('{')
        && !token.contains('}')
        && !token.contains('\\'')
        && !token.contains('"')
        && !token.contains('\\\\')
        // Shell/provider expansion must never inherit a reusable cwd-scoped grant.
        && !token.contains('$')
        && !token.contains(':')
        && !token.starts_with('~')
}
'''
text = replace_once(text, old, new, "simple shell token policy")

marker = '''    #[test]
    fn test_bash_read_only_template_rejects_parent_and_absolute_paths() {
'''
test = '''    #[test]
    fn test_shell_template_rejects_cross_platform_path_expansion() {
        let tool = BashTool::new(test_dir());
        let commands = [
            "cat $HOME/secret",
            "cat ~/secret",
            "cat C:/Windows/System32/drivers/etc/hosts",
            "cat Env:PATH",
        ];

        for command in commands {
            let profile = tool.permission_profile(&serde_json::json!({ "command": command }));
            assert!(
                profile[0]
                    .resource
                    .as_deref()
                    .unwrap()
                    .starts_with(&resource_prefix("read_only_inspection", "exact")),
                "{command} unexpectedly received a reusable template grant"
            );
        }
    }

'''
if "fn test_shell_template_rejects_cross_platform_path_expansion()" not in text:
    if text.count(marker) != 1:
        raise SystemExit("permission test insertion marker mismatch")
    text = text.replace(marker, test + marker, 1)

path.write_text(text, encoding="utf-8")

adr = Path("docs/decisions/057-windows-powershell-process-boundary.md")
text = adr.read_text(encoding="utf-8")
old = "- Permission resource prefixes and descriptions use the actual platform tool name. Unknown or complex commands remain exact resources; this decision does not grant reusable trust to PowerShell grammar."
new = "- Permission resource prefixes and descriptions use the actual platform tool name. Unknown or complex commands remain exact resources. Reusable shell templates reject variable/home expansion and colon-bearing Windows drive or PowerShell provider tokens, so cwd-scoped grants cannot authorize external paths through platform expansion."
text = replace_once(text, old, new, "ADR permission statement")
adr.write_text(text, encoding="utf-8")

review = Path("docs/reference/I170-WINDOWS-SHELL-SECURITY-REVIEW-2026-08-01.md")
text = review.read_text(encoding="utf-8")
old = "- high-risk commands do not acquire reusable permission templates merely because of the platform rename."
new = "- high-risk commands do not acquire reusable permission templates merely because of the platform rename; drive/provider paths and `$`/`~` expansion fall back to exact resources."
text = replace_once(text, old, new, "security review permission evidence")
review.write_text(text, encoding="utf-8")
