# I140 / SEC-001 Security Review

**Date**: 2026-07-17
**Status**: Accepted / Complete
**Decision**: ADR-047

## Boundary Reviewed

The change permits a file tool to leave its workspace only when a permission-aware composition
root supplies a structured authorization created after the applicable permission decision. It does
not change OS sandboxing or make an approval a general filesystem capability.

## Threat Review

| Threat | Control | Evidence |
| --- | --- | --- |
| Direct raw-tool bypass | `AgentTool::execute` supplies no authorization; external path rejected | `external_file_tools_require_exact_structured_authorization` |
| Grant reused for another file | Normalized path equality is exact | `structured_authorization_cannot_be_reused_for_another_path_or_operation` |
| Grant reused for delete/edit/write | Tool name and nature are part of the capability | same regression |
| `..` traversal | Lexical component normalization rejects escape above filesystem root | core normalization plus existing traversal tests |
| Symlink retarget / TOCTOU | Canonicalize on grant and re-normalize on execution | `structured_authorization_fails_closed_after_symlink_target_changes` |
| New write target through symlink parent | Nearest existing ancestor is canonicalized | structured write fixture |
| Deny overridden by approval/trust | Permission engine checks Deny first | `deny_rule_still_wins_for_external_path` |
| Headless accidental access | Unresolved Ask without handler returns structured denial | `external_read_without_handler_fails_closed` |
| User denial leaks content | Denial returns before tool execution | `external_read_explicit_denial_fails_closed` |
| Generic Read Allow bypasses Ask | Only a concrete path-scoped Allow can authorize external path | `external_read_path_requires_ask_not_allow` |
| Capability broadens bash/network | Only path facets produce authorization; consumers are file tools | type and composition-root inspection |
| Approval data leaks snapshot/private state | Existing `project_input` approval projection unchanged | existing projection regressions |

## Platform Review

Paths remain `PathBuf`/`Path` values; business identifiers are not converted to filenames. No
separator, drive prefix, or case behavior is implemented with string concatenation. Tests use
platform-generated temporary absolute paths and therefore exercise Windows, macOS, and Linux
path forms in their native CI environments.

Windows reparse-point behavior has the same fail-closed property as other canonicalization errors:
if normalization cannot prove the current target matches the authorization, execution is rejected.

## Residual Risk

- Filesystem state can still change after final validation and before the OS opens the path. A
  future descriptor-relative/capability-handle implementation could narrow that race further.
- In-process Rust callers can construct the additive public authorization type. They can already
  call raw tool APIs and configure composition roots; this type is not a cross-process security
  token. Talos's enforced product boundary remains the permission-aware composition roots.
- Existing “always” write scope semantics may cover the displayed directory. This change does not
  broaden or silently alter that established prompt scope; every individual execution still gets
  a tool/path-bound capability.

These residuals do not justify keeping approval non-functional. The implemented behavior is
strictly narrower than a boolean escape hatch and fails closed on uncertainty.
