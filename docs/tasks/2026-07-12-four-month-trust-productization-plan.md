# 2026-07-12 Four-Month Trust And Productization Plan

**Status**: In Progress (I116 active 2026-07-12)
**Timebox**: 2026-07-12 through 2026-11-11 (16 weeks)
**Owner area**: State truth, permission enforcement, bounded productization, Talos-primary evidence.
**Primary release posture**: Continue pre-1.0 patch/minor releases; `v1.0.0` remains conditional on REL-002.
**Supersedes**: The unexecuted remainder of the 2026-07-03 product-hardening plan and planned
iterations I086-I089. Their published objectives remain historical baselines.

## Objective

Turn the post-`v0.3.4` runtime into a trustworthy productization baseline: make governance state
match shipped code, close the workspace-trust command-execution gap without weakening permissions,
finish bounded local extension/distribution surfaces, and produce genuinely Talos-primary
self-bootstrap evidence before making any stronger release claim.

## Hard Constraints

- No global `bash`/`exec` Allow, implicit workspace trust, or Deny bypass.
- No remote dashboard access, web write route, browser automation, remote plugin install,
  marketplace, write-capable plugin tools, or executable hook carriers.
- No new native/runtime dependency without an accepted ADR and integration-boundary failure plan.
- No `v1.0.0` tag unless REL-002 is independently re-audited as fully met.
- Every month delivers a runnable operator/user result; governance-only work must directly unblock
  or verify that result.
- All workspace validation uses the pinned toolchain, committed `Cargo.lock`, and
  `./scripts/release_preflight.sh`.

## Current-State Inventory And Disposition

| State | Items | Disposition Before I116 Activation |
|---|---|---|
| Paused | I085 | Perform or explicitly continue blocking the MC107 real-terminal `/connect` walkthrough. Do not absorb it silently into I116. |
| Review | I106-I109 | Close to Complete with their recorded non-qualifying REL-002 result, or record an exact review blocker. They do not become qualifying retroactively. |
| Planned | I018-I020, I028 | Preserve/defer. I019 remains dependent on I018; I020 on I019; I028 stays gated by SCHED-001/PERM-005. |
| Planned, superseded remainder | I081-I083 | Preserve as historical shells. |
| Planned | I086-I089 | Superseded by I116-I119 because the post-v0.3.4 objectives and acceptance targets changed. |
| Complete but owner-state drift | I110-I115 deliveries; SESSION-004, PERF-001, TOOL-020, HOOK-001 records | Reconcile owner docs against code and execution evidence in I116 before selecting new feature work. |
| Partial/Planned high risk | PERM-004/PERM-005 | PERM-004 file-write trust remains bounded; PERM-005 ADR/security review precedes any command-execution trust implementation. |
| In Progress | PLUGIN-001, CMD-002, WEB-001 | Continue only their accepted local/read-only slices in I118; no scope expansion. |

## Four-Month Execution Matrix

| ID | Week | Iteration | Track | Runnable Deliverable | Validation | Initial State |
|---|---:|---|---|---|---|---|
| **Month 1 — State Truth And Operator Baseline** |||||||
| N100 | 1 | I116 | Governance | Reconcile iteration/backlog/Board state for I110-I115 and delivered SESSION-004/PERF-001/TOOL-020/HOOK-001 slices. | governance validator; code/owner trace matrix | Planned |
| N101 | 1 | I116 | Review closure | Resolve I085 and I106-I109 dispositions without upgrading non-qualifying REL-002 evidence. | terminal evidence or explicit blocker; owner-doc sync | Planned |
| N102 | 2 | I116 | Runtime smoke | Add one repeatable operator smoke packet covering version, model/connect, session export/resume, permission preflight, and ordered tool turn. | real `talos` binary; no network-required assertions | Planned |
| N103 | 3 | I116 | Diagnostics | Expose one read-only status summary that reports release/toolchain, session format, workspace trust state, and active residual gates without secrets. | CLI/TUI tests; redaction tests | Planned |
| N104 | 4 | I116 | Closeout | Month-1 truth matrix, docs, and residual owners agree with runtime. | full preflight; governance; diff check | Planned |
| **Month 2 — Command Sandbox Evidence** |||||||
| N110 | 5 | I117 | Permission design | PERM-005 ADR/security review defines access evidence, unknown access, platform limits, and safe fallback. | accepted ADR; sandbox escape review | Planned |
| N111 | 6 | I117 | Permission model | Add typed read/write/delete/spawn/network/unknown access evidence without granting authority by itself. | permission/core serialization and precedence tests | Planned |
| N112 | 6-7 | I117 | Bash/exec boundary | In trusted repos, only provably bounded command execution may use trust; unknown or out-of-repo access escalates or denies. | traversal, symlink, child-process, unknown-access, Deny tests | Planned |
| N113 | 7 | I117 | Trust controls | Provide explicit trust status and revoke UX; non-Git workspaces remain strict. | real CLI smoke; persistence and redaction tests | Planned |
| N114 | 8 | I117 | Closeout | Publish the logical-vs-OS-sandbox limitation and retain strict network/push/release gates. | full preflight; permission security review | Planned |
| **Month 3 — Bounded Local Productization** |||||||
| N120 | 9 | I118 | Extensions | Close local explicit read-only PLUGIN-001/CMD-002/HOOK-001 diagnostics with provenance and collision handling. | plugin/CLI tests; fixture smoke | Planned |
| N121 | 10 | I118 | Ingestion | Verify and finish bounded local text/HTML/JSON/CSV/Markdown extraction and fetch/save/extract handoff. | size/type/permission/failure tests | Planned |
| N122 | 10-11 | I118 | Distribution | Validate `talos.hwj.zone` installer entrypoints against canonical scripts, checksums, offline failure, and GitHub assets. | site validator; installer dry-run fixtures | Planned |
| N123 | 11 | I118 | Dashboard | Close read-only WEB-001 status/history/governance diagnostics and redaction residuals. | loopback/auth/redaction/no-write-route tests | Planned |
| N124 | 12 | I118 | Closeout | Produce a patch-release candidate only if local/read-only boundaries and installation checks pass. | full preflight; build/publish guards; docs | Planned |
| **Month 4 — Talos-Primary Evidence And Release Decision** |||||||
| N130 | 13 | I119 | Self-bootstrap | Execute at least two bounded tasks with the `talos` binary as sole primary planner/executor; external agents may observe but not author the solution. | immutable session/commit/evidence packet | Planned |
| N131 | 14 | I119 | Evidence | Capture validation, permission decisions, git status/commit linkage, failure/recovery, and issue sync without hidden manual substitution. | evidence schema validation; replay audit | Planned |
| N132 | 15 | I119 | REL-002 | Re-audit every REL-002 criterion; preserve NO-GO unless all are directly evidenced. | dated readiness report; independent trace matrix | Planned |
| N133 | 15-16 | I119 | Release | Prepare a pre-1.0 release, or `v1.0.0` only if REL-002 is fully met and explicitly approved. | release preflight; tag/version check; install smoke | Planned |
| N134 | 16 | I119 | Handoff | Final four-month matrix, architecture/permission residuals, release posture, and next owner plan. | governance; docs sync; clean worktree | Planned |

## Monthly Exit Criteria

| Month | Exit Criteria |
|---|---|
| 1 | Board/backlog/iteration state matches code; stale reviews have dispositions; operator smoke and status summary run without secrets or network dependency. |
| 2 | Trusted workspace file behavior remains intact; bash/exec trust is evidence-based; unknown/out-of-repo access cannot silently inherit trust; explicit revoke works. |
| 3 | Local plugin/hook/document/dashboard/install surfaces are useful and test-backed without remote/write/browser scope expansion. |
| 4 | Talos-primary evidence is reproducible; release decision is evidence-based; residuals and next owners are explicit. |

## Dependency Order

```text
N100/N101 -> N102/N103 -> N104
N104 -> N110 -> N111 -> N112/N113 -> N114
N114 -> N120/N121/N122/N123 -> N124
N124 -> N130 -> N131 -> N132 -> N133/N134
```

## Risk And Fallback Register

| Risk | Response | Safe Fallback |
|---|---|---|
| PERM-005 cannot observe arbitrary command access portably | Timebox ADR and prototype; distinguish declared, observed, and unknown evidence. | Keep bash/exec per-command Ask/Deny and close only diagnostics/revoke UX. |
| State reconciliation reveals incomplete implementation | Correct owner status before planning code. | Mark Partial/Blocked with exact test or runtime gap. |
| Extension work grows into marketplace/remote install | Enforce local explicit manifests and existing ADRs. | Ship diagnostics only. |
| Document ingestion requires heavy/native parsers | Stop for ADR/dependency review. | Keep bounded text/HTML/JSON/CSV/Markdown extraction. |
| Site installers cannot be validated end-to-end | Do not change default install command. | Keep GitHub Releases/raw repository scripts as source of truth. |
| Talos-primary tasks require external agent authorship | Classify evidence as non-qualifying. | Keep REL-002 NO-GO and publish only pre-1.0 if otherwise approved. |

## Activation And Recovery

- I116 is Active after the non-terminal inventory received recorded dispositions. Execution and
  recovery are owned by `docs/tasks/2026-07-12-developer-trust-productization-long-task.md`.
- Activate one iteration at a time. Later months remain Planned until the prior month closes.
- Resume by reading this document, the current iteration, `docs/BOARD.md`, and each selected owner
  story. Update owner docs before derived views.
- Every closeout runs `./scripts/release_preflight.sh`, governance validation, and `git diff --check`.

## Documentation Owners

- `docs/iterations/I116-state-truth-operator-baseline.md`
- `docs/iterations/I117-command-sandbox-evidence.md`
- `docs/iterations/I118-bounded-local-productization.md`
- `docs/iterations/I119-talos-primary-release-decision.md`
- `docs/backlog/PRODUCT-BACKLOG.md`, `docs/BOARD.md`, ADR index, README/install docs as affected

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-12 | Plan publication | New I116-I119 sequence created after inventorying current non-terminal work. No iteration activated and no high-risk implementation authorized. |
| 2026-07-12 | I116 activation | I085 remains explicitly Paused for MC107 terminal evidence; I106-I109 closed Complete with non-qualifying REL-002 classifications preserved. I116 activated and the developer long-task contract published. No I117 permission implementation, push, or release action authorized. |
