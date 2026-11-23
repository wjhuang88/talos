# GIT-ENV-001 — Scoop installation environment and prepared Git writes

Status: Active / Claimed proposed, effective only on main. Source Issue: [#1004](https://github.com/wjhuang88/talos/issues/1004).

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6.2 Astra |
| Work Slice | GIT-ENV-001 Windows Scoop installation-root admission for prepared Checkout and Commit under ADR-089, including native execution and refusal evidence. I324 owns unsupported host queries. |
| Claimed At | 2026-11-23 |
| Source Issue | #1004 |
| Governance Claim PR | Pending |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer requested dedicated handling. Independent approval and CI required before main activation. |
| Implementation PR | Pending effective main claim |
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

On Windows x64, Rust 1.99.0 MSVC at main `1b3fc18b`, the inherited Git 2.36.1 environment produced 175 workspace failures, including 144 with an override refusal. Removing only the install-root variable in the test process produced 135 failures and zero such refusals. With isolated MinGit 2.56.1 the corresponding figures were 108 and zero. The last two runs overlapped. These counts do not attribute every remaining failure. The Windows classifier fixture passed 15 tests with Git index modes. The three historical raw captures shipped with intake correspond to the [November 10 report](../../reference/WINDOWS-VALIDATION-2026-11-10.md).

## Required Reads

[ADR-089](../../decisions/089-structured-git-effect-authorization.md), [I324 owner](GIT-COMPAT-001-gix-preflight-host-git-compatibility.md), [collaboration SOP](../../sop/AGENT-COLLABORATION.md). A target-branch claim and reviewed authorization decision precede production changes.
