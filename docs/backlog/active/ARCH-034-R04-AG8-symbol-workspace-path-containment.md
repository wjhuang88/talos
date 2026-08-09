# ARCH-034-R04-AG8: Symbol Workspace Path Containment

| Field | Value |
|---|---|
| Parent | ARCH-034-R04 |
| Finding | PR #177 review residual 3 / symbol read-path workspace escape |
| Status | Refinement — lexical, canonical and symlink-root policy require security review |
| Priority | P0 |
| Selected Iteration | None |
| Preserved behavior | Symbol result schemas/order, traversal budgets/notices, language detection, read-only classification, and valid in-workspace relative paths |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Not applicable |
| Authorization Mode | Independent security review required |
| Authorization Evidence | PR #177 independent review comment `5230395611` confirmed the pre-existing gap; no implementation authorization. |
| Implementation PR | Not started |
| Last Updated | 2026-08-09 |
| Handoff / Release Condition | Decide the workspace read boundary, establish an effective claim, and select a dedicated iteration after I182 closes. |

## Confirmed Baseline

`execute_find_symbol` and `execute_list_symbols` join caller input directly onto
`workspace_root`. An absolute path replaces the base and parent components are
not normalized, so the read-only tools can address paths outside the workspace.
The same policy question applies to an in-workspace root symlink targeting an
external directory. The gap predates I182 and is outside AG-4's public-input
Non-Goals.

## Scope And Acceptance

- Select one canonical workspace-read containment contract shared by both symbol
  tools and compatible with the existing permission architecture.
- Decide absolute paths, `..`, nonexistent targets, lexical aliases, root
  symlinks and descendant symlinks explicitly; do not infer security from
  `PathBuf::starts_with` alone.
- Reject an escaping request before filesystem traversal or parser admission and
  return one stable, redacted error without exposing external file contents.
- Preserve valid relative in-workspace requests and AG-4's directory traversal
  budgets/results byte-for-byte.
- Add Unix symlink-escape and cross-platform absolute/parent-component fixtures,
  plus focused permission/read-only classification regressions.
- Review whether the shared workspace path resolver is sufficient before adding
  a symbol-specific implementation.

## Exclusions And Residuals

No write permission change, broad read-permission redesign, schema change,
filesystem sandbox, canonicalization of every result, or AG-4 counter change.
AG-10 owns notice semantics and AG-9 owns invalid-text parity.

## Minimum Validation

Focused `talos-tools` tests, permission regressions, locked release preflight,
Unix/Windows CI, both governance validators and independent security review.
