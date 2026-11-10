# Native Windows validation — 2026-11-10

## Scope and provenance

Native Windows / PowerShell execution against source commit
`c065236a1960e45920e9732212003f92bdefc57b` (PR #903 consolidation).
Compiler verified as Rust 1.99.0, host `x86_64-pc-windows-msvc`;
system Git reports `2.36.1.windows.1`. These are local observations, not remote CI
or independent security review. No product code was changed in this validation session.

## Commands and observed results

All test commands used `cargo +1.99.0-x86_64-pc-windows-msvc test --locked`.

| Remaining arguments | Observed result | Evidence boundary |
|---|---|---|
| `-p talos-tui --lib` | Exit 0; 652 passed, 0 failed, 0 ignored | Includes `compact_approval_preserves_localized_review_effects_and_decisions`, compact/paginated large approval and scope rendering regressions; no real-terminal manual acceptance |
| `-p talos-tools --features shell --lib process_boundary` | Exit 0; 58 passed, 0 failed, 0 ignored, 104 filtered | Native Job Object assignment, termination, descendant cleanup, failure/handle ownership and foreground I/O tests; does not close TEST-003 unknown-resource residual |
| `-p talos-skill --lib` | Exit 0; 71 passed, 0 failed, 0 ignored | Existing suite; no Windows junction/directory-symlink acceptance established |
| `-p talos-tools --features git --lib` (inherited environment) | Exit 1; 259 passed, 113 failed, 0 ignored | Prepared operations can refuse inherited `GIT_INSTALL_ROOT`; subsequent failures need independent classification |
| Same Git command, removing `GIT_INSTALL_ROOT` and `GIT_PAGER` only in the invoking test process | Test result FAILED; 292 passed, 80 failed, 0 ignored | Diagnostic controlled environment only; not a supported product workaround or compatibility pass |
| `-p talos-tools --features git --lib large_tracked -- --nocapture` | Exit 1; 0 passed, 1 failed, 371 filtered | `large_tracked_source_edit_after_preparation_blocks_checkout` refused on Git environment overrides; snapshot acceptance not established |

The first process-boundary command omitted the `shell` feature and ran **zero tests**
(40 filtered). It is not validation evidence; the 58-test feature-enabled rerun above
supersedes it. GNU-toolchain startup attempts and concurrent rustup recovery failed;
only actual MSVC test executions above count. A working MSVC compiler was verified,
but the separate recovery command's failure does not prove full toolchain repair.

Raw local captures are `windows-tools-validation.log`, `windows-process-validation.log`
and `windows-skill-validation.log` at the workspace root. They are untracked diagnostic
artifacts, not committed evidence artifacts. The tools capture contains the controlled
environment rerun. TUI and initial/focused Git results were observed in session command
output; this report records them without inventing persistent raw captures.

## Git failure diagnosis and limitations

`git var GIT_ATTR_SYSTEM` and `git var GIT_ATTR_GLOBAL` both return usage on this
Git installation. Prepared discovery calls these before checkout/commit and refuses
when it cannot discover the required attribute paths. This is an unhandled integration
capability assumption; actual checkout/commit support does not imply these queries exist.
Other failure examples include URL rewrite and tracking-overlay comparisons. The 80
failures are not all proven to share a single cause.

`GIT_PAGER` is explicitly allowed by current validation; its removal in the diagnostic
rerun must not be interpreted as proof that it caused rejection. `GIT_INSTALL_ROOT`
matches the broad rejection rule. No global/user environment or Git configuration was
changed to bypass that rule.

## Retained acceptance

- TUI automated native tests passed; real Windows terminal interaction, layout/resizing
  and actual user approval walkthrough remain unverified.
- Git snapshot functionality is not accepted on this Git installation; resolve compatibility
  and rerun the complete Git-enabled suite and large-content regressions.
- TEST-003/#717 unknown/untracked native lifetime remains open; the passing process suite
  does not prove absence of all leaks or establish a legitimate-cache explanation.
- Windows Skill junction/symlink and Desktop native interaction remain unverified.
- No workspace-wide preflight, exact-head remote CI, independent review or iteration
  completion is claimed by this report.

The maintainer selected continued Git 2.36 support. The proposed implementation boundary
is recorded in [gix preflight compatibility](../proposals/gix-preflight-system-git-compatibility.md);
it remains unimplemented.

## Documentation closeout checks

Staged diff review and `git diff --cached --check` passed for this documentation
follow-up. Native `scripts/validate_project_governance.ps1 .` exited 1: the SQLite
consumer and delivery-workflow Python subchecks failed. `python`/`python3` resolve
to Windows Store application aliases; no usable Python interpreter was found on
the inspected PATH. Shell `scripts/validate_project_governance.sh .` also exited 1
with three Python-subcheck errors (SQLite consumers, delivery workflow, remote Issue
owner regressions). These are not passing governance results.
`bash scripts/validate_collaboration_claims.sh .` passed with zero warnings.
No runtime retest is implied by documentation-only changes.
