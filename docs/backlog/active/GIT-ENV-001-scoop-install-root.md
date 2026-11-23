# GIT-ENV-001 — Scoop installation environment and prepared Git writes

Status: Complete / Closed for Scoop environment admission. Source Issue: [#1004](https://github.com/wjhuang88/talos/issues/1004).

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6.2 Astra |
| Work Slice | GIT-ENV-001 Windows Scoop installation-root admission for prepared Checkout and Commit under ADR-089, including native execution and refusal evidence. I324 owns unsupported host queries. |
| Claimed At | 2026-11-23 |
| Source Issue | #1004 |
| Governance Claim PR | #1006 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer requested dedicated handling. Independent approval and CI required before main activation. |
| Implementation PR | #1007 (merged) |
| Last Updated | 2026-11-23 |
| Handoff / Release Condition | Reviewed permission decision, native default Scoop approval/denial/stale tests and implementation merge evidence. |

The installed Scoop shim points to the underlying installation's cmd/git.exe. Discovery and revalidation both call validate_environment in prepared_add.rs. Review both call sites and executable identity handling together. The global environment-variable value alone cannot establish executable identity.

Scoop sets `GIT_INSTALL_ROOT` to the installation directory. With default Windows settings the prepared writer currently rejects the environment before Checkout can be approved. Users need approved operations to work while keeping repository, index, attributes, helpers and executable identity bound to the prepared plan.

## Acceptance

- Exercise prepared Checkout and Commit with the inherited default Scoop setting on native Windows. Verify resulting branch, tree and index effects, including unrelated staged content, and preserve denial and stale-plan checks.
- Establish how the shim and selected Git interpret the variable before deciding whether to omit, validate or retain it. Check the same environment at discovery and execution. Exercise mismatched installation identity and keep repository/index/config redirects subject to existing authorization.
- Record independent permission/security review against ADR-089, exact source CI, native Git versions and sanitized outcomes. Update the Git user documentation with supported installation behavior.
- Coordinate unsupported Git 2.36 query semantics with #919 / #928 / #931. A query substitution and a change in environment admission are separate decisions.

## Validation baseline

### Implementation checkpoint after activation

Intake #1005 merged at `5a30a671`. Claim #1006 merged at `b4abc714` after independent review and applicable CI. The implementation worktree starts at that merge.

Independent design review identified two authorization stages for the installed Scoop layout. The program selected on PATH is the shim and its sibling mapping names the Git executable. Reading that mapping requires exact Read admission. Binding and launching the mapped executable requires its own Read and Execute facets. Any design using this mapping must bind the original setting, mapping and program identities through execution, reject unsupported mapping directives, and check the association again before writing. Recorded read-only native comparisons do not establish equivalence for approved writes. Product implementation and native acceptance remain open.

On Windows x64, Rust 1.99.0 MSVC at main `1b3fc18b`, the inherited Git 2.36.1 environment produced 175 workspace failures, including 144 with an override refusal. Removing only the install-root variable in the test process produced 135 failures and zero such refusals. With isolated MinGit 2.56.1 the corresponding figures were 108 and zero. The last two runs overlapped. These counts do not attribute every remaining failure. The Windows classifier fixture passed 15 tests with Git index modes. The three historical raw captures shipped with intake correspond to the [November 10 report](../../reference/WINDOWS-VALIDATION-2026-11-10.md).

## Native implementation evidence

The implementation candidate binds the mapping and selected executable to exact Read and Execute facets for prepared Checkout/Commit. The mapping guard checks a 4 KiB bound and retains Windows file identity. The raw setting is checked again before execution.

On Windows x64 with Rust 1.99.0 MSVC, the installed Scoop launcher and a workspace-local Scoop-layout PortableGit 2.56.1 installation passed the 17 MiB create-branch, commit and restore test with GIT_INSTALL_ROOT set. The stale tracked-source case, two mapping tests, and the Runtime conditional Checkout approval/denial test passed. On the machine's installed Scoop Git 2.36.1 the same 17 MiB test advanced to the unsupported attribute query owned by #919. Keep that version-specific acceptance open. MinGit 2.56.1 also lacks the internal git.exe expected under its reported exec-path.

## Required Reads

[ADR-089](../../decisions/089-structured-git-effect-authorization.md), [I324 owner](GIT-COMPAT-001-gix-preflight-host-git-compatibility.md), [collaboration SOP](../../sop/AGENT-COLLABORATION.md). A target-branch claim and reviewed authorization decision precede production changes.

## Completion evidence

Completion Commit: f4eb947c2f77e8d8ed124b16fdb21e177a692561

[Product PR #1007](https://github.com/wjhuang88/talos/pull/1007) merged after [all eight checks](https://github.com/wjhuang88/talos/actions/runs/45132476825) passed and [independent permission review](https://github.com/wjhuang88/talos/pull/1007#issuecomment-7095678188) approved c1b0e952. Final native coverage includes index preservation and Runtime denial. On the installed Git 2.36.1 the remaining unsupported attribute query belongs to #919.
