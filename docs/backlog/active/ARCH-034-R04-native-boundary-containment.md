# ARCH-034-R04: Native And Panic-Boundary Containment

| Field | Value |
|---|---|
| Parent | ARCH-034 |
| Findings | ARCH-034-F13, ARCH-034-F19 |
| Status | Partial - I181 review Complete; accepted gaps require child owners/claims |
| Priority | P1 |
| Selected Iteration | I181 (Complete; review evidence `aea26ad0`) |
| Preserved behavior | Permission gates, native error mapping, process limits, storage format, and fallback policy |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-08 |
| Work Slice | Independently review the recorded native/panic boundary matrix, classify each gap as accepted, rejected, or requiring more evidence, reconcile ADR-007/ADR-008/ADR-020 facts, and define separately claimable implementation slices; no production, test, dependency, permission, sandbox, process-hardening, `unsafe`, or policy change. |
| Claimed At | 2026-08-08 |
| Source Issue | None |
| Governance Claim PR | #174 |
| Authorization Mode | Independent review |
| Authorization Evidence | PR #174 review `PRR_kwDOSrj_LM8AAAABI2KjFw` independently analyzed `24694b88` and approved the review-only plan subject to mandatory corrections. Maintainer clarification [comment `5225938556`](https://github.com/wjhuang88/talos/pull/174#issuecomment-5225938556) attests that a different natural person performed the review through the shared @wjhuang88 credential, accepts the content as independent-review evidence, and preserves GitHub's actual `COMMENTED` state. Commit `f263f221` incorporates the required corrections; exact-head CI `31253291503` passed. |
| Implementation PR | None - review-only claim merged; protected implementations require child owners |
| Last Updated | 2026-08-08 |
| Handoff / Release Condition | Parent claim coordinates creation of non-overlapping child owners only. I181 review completed at `aea26ad011af04396ab8588c9326d309538f31a2`; do not implement any accepted gap under this review-only claim. |

## Problem And Boundary

`gix`, arborium/tree-sitter, bundled SQLite, subprocess spawning, and ADR-007 libc sites do not have
one current containment matrix. Some boundaries use timeout, error propagation, or `catch_unwind`;
coverage must be proven per call family rather than assumed.

## Scope

- Produce a call-site/failure-mode/containment/test matrix for every native or panic-capable boundary.
- Reconcile ADR-007 and R0 status facts without weakening their restrictions.
- Route each accepted gap to a separate bounded owner/claim before adding narrow containment/tests.

## Pre-Review Boundary Matrix

This matrix is read-only review input, not a security verdict or implementation authorization.
Locations and current behavior are confirmed from `main` at `46c65805`; risk classifications and
candidate remedies were reviewed through PR #174, with the resulting disposition recorded below.

| Boundary family | Confirmed call sites | Current containment | Review question / candidate gap | Required evidence before disposition |
|---|---|---|---|---|
| ADR-007 parent hardening | `talos-sandbox/src/hardening.rs` | `setrlimit` return values become `SandboxError`; the public parent-environment mutation API has no production caller, while production callers use only side-effect-free `dangerous_env_var_names()` | The current product does not exercise the parent-mutation race, but the semver-bound public `&self` API remains a latent trap that cannot enforce the before-threads invariant; its safety comment also incorrectly claims mutable access. | Independent soundness/API review plus a controlled concurrency/call-order fixture; no parent-process mutation change in I181. |
| Bash Unix `pre_exec` | `talos-tools/src/bash_tool.rs` | Child environment removal and three rlimits run before exec; spawn/wait errors and an absolute direct-child timeout are mapped | All `unsetenv`/`setrlimit` return values are ignored and the closure always returns `Ok(())`, so native failure silently execs a less-hardened child. Neither function appears in the normative POSIX async-signal-safe list; the current `unsetenv` claim is false and has a post-fork locking risk. ADR-007 also still labels this shipped site as planned. | Escape-vector review, forced-failure strategy, child-limit/env fixtures, a fail-closed fallback, and an independently reviewed ADR-007 correction. |
| Direct `exec` subprocesses | `talos-tools/src/exec_tool.rs` | Permission facets, child-local environment scrub, spawn/wait error mapping, bounded output, and a direct-child timeout | After timeout, the direct child is killed but the code unconditionally awaits stdout/stderr readers. A descendant retaining inherited pipe fds can prevent EOF indefinitely, so the advertised timeout is not an operation deadline. Neither bash nor exec proves process-tree cleanup. Unix rlimit parity remains a separate behavior-affecting policy decision. | A grandchild-held-pipe fixture that returns within deadline plus margin, partial-output preservation, process-tree cleanup evidence, and a separately reviewed decision if rlimit parity is pursued. |
| Other runtime subprocesses | `talos-sandbox/src/lib.rs`, `talos-mcp/src/client/transport.rs`, `talos-tools/src/git.rs`, `git_write.rs`, `read_image_tool.rs`, `talos-tui/src/clipboard.rs`, `talos-conversation/src/validation.rs` | Error propagation is present in most families; timeout, environment scrub, kill-on-drop, output bounds, and permission ownership vary by caller | Classify which calls execute untrusted/user-selected programs and therefore need operation deadlines, environment controls, process-tree cleanup, or permission proof. Build-time `talos-config/build.rs` curl remains a separate build-boundary review item. | Per-family caller/authority map and controlled spawn-not-found, nonzero, timeout, inherited-env, and cleanup fixtures. |
| Arborium/tree-sitter | Four parser construction/language/parse paths in `talos-tools/src/symbol.rs`; `talos-tui/src/highlight.rs` | Tool language/parse absence degrades to `None` or `ToolResult::error`; TUI highlighting catches panics and falls back after a completed call reports more than 500ms elapsed | Tool parser work runs synchronously inside async execution with no visible `catch_unwind` or deadline. The TUI elapsed check cannot interrupt a hung/long native call. Both recursive tool walks follow directory symlinks without cycle/depth guards, so a cycle can cause an uncatchable stack-overflow abort; files are read without a byte cap before parsing. | Both crates in the reverse dependency tree; symlink-cycle and oversized-file fixtures, depth/count/byte/deadline limits with explicit truncation, malformed/adversarial corpus, narrow panic injection, and proof of plain-text/error fallback without process exit. |
| `gix` read operations | `talos-tools/src/git.rs`, repository discovery used by `git_write.rs`, and `talos-tui/src/scrollback_status_git.rs` | Most tool results map to `GitToolError`; one host-git diff path has a 10-second timeout. TUI status catches panics, maps failures to no summary, and caches for 500ms | Tool in-process discovery/status/revision/walk/reference calls have no panic boundary or operation timeout and execute synchronously in async tools. TUI containment prevents panic exit but has no operation deadline; cache cadence is not a timeout. Host-git write calls also lack a deadline. | Both crates in the reverse dependency tree; corrupt repository, hostile ref/object, large-walk, panic-containment, and timeout fixtures; confirm whether blocking work must move behind a bounded adapter. |
| Bundled SQLite | `talos-session`, `talos-evolution`, `talos-exploration`, `talos-memory`, and `talos-models` | `rusqlite::Error` is generally propagated; busy timeout/retry behavior exists only in selected session paths | No visible `catch_unwind` boundary exists. Busy/locked/corrupt handling and operation deadlines are inconsistent. ADR-008 says the exception is limited to two crates, while five manifests currently enable `rusqlite/bundled`; this policy fact must be reconciled, not silently broadened. | Five-crate call-family inventory; corrupt/busy/locked/migration/panic fixtures; explicit ADR disposition before any policy or dependency edit. |
| Existing panic adapters | `talos-tools/src/image_validation.rs`, `search_engine.rs`; `talos-tui/src/highlight.rs`, `scrollback_status_git.rs`, and Mermaid rendering | Selected image/search/TUI dependency calls already use `catch_unwind` and safe fallback | Use these only as local evidence that narrow adapters are possible; panic containment does not itself provide an operation timeout, and no catch-all panic swallow may cover unrelated logic. | Reviewer-approved adapter boundary and tests that distinguish dependency panic from Talos logic defects. |

## Pre-Review Findings

- **Confirmed facts:** five production `unsafe` lexical candidates remain; `ProcessHardening::apply`
  has no production caller but exposes a latent public parent-mutation trap; bash hardening ignores
  native return values and relies on a false POSIX async-signal-safety claim; direct exec can wait
  forever on descendant-held pipe fds after its timeout; symbol traversal follows directory
  symlinks without cycle/depth guards and reads files without a byte cap; symbol and in-process
  `gix` paths have no enforceable deadline adapter; five crates enable bundled SQLite while ADR-008
  names two; ADR-007 still describes the shipped `pre_exec` site as planned.
- **Content authorization:** PR #174 review `PRR_kwDOSrj_LM8AAAABI2KjFw` independently reproduced
  these facts and proposed the dispositions below. GitHub records it as `COMMENTED` by the shared
  @wjhuang88 credential; maintainer comment `5225938556` attests that a different natural person
  performed the review and accepts its content as independent-review evidence without representing
  the event as a native GitHub `APPROVED` review.
- **Prohibited conclusion:** I181 must not mark R04 remediated or authorize production changes. It
  may only produce an independently approved disposition and bounded follow-up owners.

### Dependency And Test Trace

- `cargo tree --locked -i arborium-tree-sitter` confirms production consumers in `talos-tools` and
  `talos-tui`; `cargo tree --locked -i gix` confirms the same two consumers.
- `cargo tree --locked -i libc@1.0.0-alpha.3` confirms direct use by `talos-sandbox` and
  `talos-tools`.
- `cargo tree --locked -i libsqlite3-sys` confirms one shared native SQLite version reached by
  `talos-session`, `talos-evolution`, `talos-exploration`, `talos-memory`, and `talos-models`.
- Existing positive/failure coverage includes bash/exec direct-child timeouts, Unix child
  environment/core-limit checks, TUI non-repository and normal repository status, selected
  session corrupt/busy SQLite cases, and a corrupt models catalog. The review must not generalize
  those focused tests into proof for untested panic, deadline, native-return, or five-crate SQLite
  families.

### Review Disposition

These content-reviewed dispositions are final for I181's review-only scope. They authorize claim
merge and creation of separately governed follow-up owners only; they do not authorize protected
implementation on this branch.

| Boundary family | Disposition | Candidate follow-up boundary |
|---|---|---|
| ADR-007 parent hardening | Accepted gap, latent public-API risk | AG-2: fence or narrow the unused parent-mutation API, correct its safety comment, retain `dangerous_env_var_names()`, and require sandbox/API review. |
| Bash Unix `pre_exec` | Accepted gap | AG-1: remove post-fork environment mutation where possible, fail closed on native errors, correct ADR-007, add Unix failure fixtures, and require unsafe/security review. |
| Direct `exec` subprocesses | Accepted liveness gap; rlimit parity undecided | AG-3: bound or abort pipe-reader joins after timeout while preserving partial output; keep rlimit parity in a separate behavior decision. |
| Other runtime subprocesses | Needs evidence; `git_write.rs` accepted deadline gap | Build a caller/authority map before classifying other families; include bounded non-interactive host-git execution in AG-6. |
| Arborium/tree-sitter | Accepted traversal, panic, and deadline gaps | AG-4: contain symlink/depth/count/byte traversal; AG-5: narrow parser panic/deadline adapter with plain-text/error fallback. |
| `gix` read operations | Accepted tools/deadline gaps; TUI panic adequately contained | AG-6: one bounded blocking/panic adapter plus host-git timeout; preserve error and display contracts. |
| Bundled SQLite | Accepted ADR-008 scope gap; other containment needs evidence | AG-7: separately reviewed ADR-only reconciliation plus a sixth-consumer validator; inventory all five call families before code changes. |
| Existing panic adapters | Adequately contained | Retain narrow dependency-only scope; do not reuse dependency state after panic without an explicit reset contract. |

## Exclusions

- No sandbox, permission, process-hardening, `unsafe`, dependency, or policy edit in I181; each
  accepted implementation gap requires its own bounded claim and applicable security review.
- No catch-all panic swallowing, silent fallback, or replacement of ADR-recorded dependencies.

## Readiness And Acceptance

- Independent review content and maintainer attestation record escape-vector and failure-mode
  analysis without overstating the GitHub review state.
- Each accepted gap has one bounded implementation slice and explicit safe fallback.
- Process, permission, git, symbol, SQLite, and crash tests cover the reviewed boundary.
- Locked workspace, platform, security, governance, and ADR checks pass.

## Rollback / Residual

I181 review evidence is merged at `aea26ad011af04396ab8588c9326d309538f31a2`. AG-1 through
AG-7 require separate bounded owners/claims; other subprocess and five-crate SQLite containment
remain evidence work until classified. New native dependencies require a separate ADR.

## Review Completion Evidence

- Review-only Completion Commit: `aea26ad011af04396ab8588c9326d309538f31a2`
- PR #174 exact head `12963970ab5ce8e316a9a649503bc220d48dff89` passed CI
  `31255683335`, both governance validators, architecture audit, scale assessment, and merge-time
  CAS before squash merge.
- This evidence completes the disposition phase only. R04 remains Partial until accepted child
  gaps are implemented or explicitly dispositioned with their own completion evidence.
