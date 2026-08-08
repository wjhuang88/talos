# ARCH-034-R04: Native And Panic-Boundary Containment

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Findings | ARCH-034-F13, ARCH-034-F19 |
| Status | Planned - I181 independent security review claim pending |
| Priority | P1 |
| Selected Iteration | I181 (Planned; claim pending) |
| Preserved behavior | Permission gates, native error mapping, process limits, storage format, and fallback policy |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Independently review the recorded native/panic boundary matrix, classify each gap as accepted, rejected, or requiring more evidence, reconcile ADR-007/ADR-008/ADR-020 facts, and define separately claimable implementation slices; no production, test, dependency, permission, sandbox, process-hardening, `unsafe`, or policy change. |
| Claimed At | Not applicable |
| Source Issue | None |
| Governance Claim PR | Pending |
| Authorization Mode | Independent review required |
| Authorization Evidence | Pending independent reviewer approval of the finalized exact-head claim PR. |
| Implementation PR | Not started |
| Last Updated | 2026-08-08 |
| Handoff / Release Condition | Do not merge the claim or begin protected implementation without independent approval; after review, release rejected gaps and create one bounded owner/claim per accepted implementation slice. |

## Problem And Boundary

`gix`, arborium/tree-sitter, bundled SQLite, subprocess spawning, and ADR-007 libc sites do not have
one current containment matrix. Some boundaries use timeout, error propagation, or `catch_unwind`;
coverage must be proven per call family rather than assumed.

## Scope

- Produce a call-site/failure-mode/containment/test matrix for every native or panic-capable boundary.
- Reconcile ADR-007 and R0 status facts without weakening their restrictions.
- After independent review, add only the narrow containment/tests required by proven gaps.

## Pre-Review Boundary Matrix

This matrix is read-only review input, not a security verdict or implementation authorization.
Locations and current behavior are confirmed from `main` at `46c65805`; risk classifications and
remedies remain pending independent review.

| Boundary family | Confirmed call sites | Current containment | Review question / candidate gap | Required evidence before disposition |
|---|---|---|---|---|
| ADR-007 parent hardening | `talos-sandbox/src/hardening.rs` | `setrlimit` return values become `SandboxError`; environment removal mutates the parent process | The public `&self` API cannot enforce the documented before-threads invariant, and its safety comment incorrectly claims mutable access. Confirm whether the parent mutation API must be removed, made startup-only by construction, or otherwise fenced. | Independent soundness review plus a controlled concurrency/call-order fixture; no parent-process mutation change in I181. |
| Bash Unix `pre_exec` | `talos-tools/src/bash_tool.rs` | Child environment removal and three rlimits run before exec; spawn/wait errors and an absolute direct-child timeout are mapped | All `unsetenv`/`setrlimit` return values are ignored and the closure always returns `Ok(())`. The comments assert POSIX async-signal-safety; the reviewer must verify that assertion against the supported libc/platform contract rather than inherit it from ADR text. | Escape-vector review, forced-failure strategy, child-limit/env fixtures, and a reviewed fallback that never silently runs less hardened. |
| Direct `exec` subprocesses | `talos-tools/src/exec_tool.rs` | Permission facets, child-local environment scrub, spawn/wait error mapping, bounded output, and direct-child timeout | Direct exec does not apply the Unix bash rlimits. Confirm whether this is an intended product distinction or a hardening gap; pipe timeout cancellation/child cleanup also needs explicit proof. | Unix/Windows direct execution tests, timeout cleanup fixture, and a documented process-limit policy shared or intentionally distinct from bash. |
| Other runtime subprocesses | `talos-sandbox/src/lib.rs`, `talos-mcp/src/client/transport.rs`, `talos-tools/src/git.rs`, `git_write.rs`, `read_image_tool.rs`, `talos-tui/src/clipboard.rs`, `talos-conversation/src/validation.rs` | Error propagation is present in most families; timeout, environment scrub, kill-on-drop, output bounds, and permission ownership vary by caller | Classify which calls execute untrusted/user-selected programs and therefore need operation deadlines, environment controls, process-tree cleanup, or permission proof. Build-time `talos-config/build.rs` curl remains a separate build-boundary review item. | Per-family caller/authority map and controlled spawn-not-found, nonzero, timeout, inherited-env, and cleanup fixtures. |
| Arborium/tree-sitter | Four parser construction/language/parse paths in `talos-tools/src/symbol.rs` | Language/parse absence degrades to `None` or `ToolResult::error`; file IO errors are mapped | Synchronous native parser work runs directly inside async tool execution with no visible `catch_unwind` or timeout, contrary to ADR-020's explicit fallback requirement. Recursive workspace traversal is also unbounded by file count/size/deadline. | Malformed/adversarial corpus, panic injection or adapter seam, deadline fixture, and proof of a plain-text/error fallback without process exit. |
| `gix` read operations | `talos-tools/src/git.rs` and repository discovery used by `git_write.rs` | Most `gix` results map to `GitToolError`; one host-git diff path has a 10-second timeout | In-process `gix` discovery/status/revision/walk/reference calls have no visible panic boundary or operation timeout and execute synchronously in async tools. Host-git write calls also lack a deadline. | Corrupt repository, hostile ref/object, large-walk, panic-containment, and timeout fixtures; confirm whether blocking work must move behind a bounded adapter. |
| Bundled SQLite | `talos-session`, `talos-evolution`, `talos-exploration`, `talos-memory`, and `talos-models` | `rusqlite::Error` is generally propagated; busy timeout/retry behavior exists only in selected session paths | No visible `catch_unwind` boundary exists. Busy/locked/corrupt handling and operation deadlines are inconsistent. ADR-008 says the exception is limited to two crates, while five manifests currently enable `rusqlite/bundled`; this policy fact must be reconciled, not silently broadened. | Five-crate call-family inventory; corrupt/busy/locked/migration/panic fixtures; explicit ADR disposition before any policy or dependency edit. |
| Existing panic adapters | `talos-tools/src/image_validation.rs` and `search_engine.rs` | Selected image/search dependency calls already use `catch_unwind` and safe error fallback | Use these only as local evidence that narrow adapters are possible; do not introduce a catch-all panic swallow around unrelated logic. | Reviewer-approved adapter boundary and tests that distinguish dependency panic from Talos logic defects. |

## Pre-Review Findings

- **Confirmed facts:** five production `unsafe` lexical candidates remain; bash hardening ignores
  native return values; symbol and in-process `gix` paths have no visible panic/deadline adapter;
  five crates enable bundled SQLite while ADR-008 names two; direct exec scrubs environment but
  does not apply bash's Unix rlimits.
- **Inference pending review:** each difference above may be a vulnerability or policy defect, but
  its severity and correct remediation are unknown until the independent reviewer validates the
  failure mode and fallback.
- **Prohibited conclusion:** I181 must not mark R04 remediated or authorize production changes. It
  may only produce an independently approved disposition and bounded follow-up owners.

## Exclusions

- No sandbox, permission, process-hardening, `unsafe`, dependency, or policy edit before security review.
- No catch-all panic swallowing, silent fallback, or replacement of ADR-recorded dependencies.

## Readiness And Acceptance

- Independent reviewer records escape-vector and failure-mode analysis.
- Each accepted gap has one bounded implementation slice and explicit safe fallback.
- Process, permission, git, symbol, SQLite, and crash tests cover the reviewed boundary.
- Locked workspace, platform, security, governance, and ADR checks pass.

## Rollback / Residual

If independent review is unavailable, remain Refinement and do not edit protected code. New native
dependencies require a separate ADR.
