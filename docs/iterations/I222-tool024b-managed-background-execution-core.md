# Iteration I222: TOOL-024-B Managed Background Execution Core

> Document status: Complete / Closed — implementation merged via PR #382; owner-first closeout recorded
> Published plan date: 2026-08-23
> Planned objective: implement ADR-060's bounded Unix Agent/session-owned background execution core
> without starting TOOL-024-C/D or overlapping I213 Dashboard/CLI production authority.
> Baseline rule: preserve this target; a changed objective requires a new iteration ID.
> MVP deliverable: a runnable Agent/session integration harness starts an explicitly admitted Unix
> background shell/single-exec job, returns a bounded receipt promptly, emits one terminal event and
> reaps the full process group under timeout/cancel/shutdown while Windows fails before spawn.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline Issue #59 session 2026-08-23 |
| Work Slice | Implement only TOOL-024-B under Accepted ADR-060: Unix Agent/session-owned supervisor, semantic pre-admission, exact background permission, bounded receipt/output/state/event, checked same-process-group cancellation/reap, and ordinary Session plus Runtime finalizer cleanup. Permission authority is limited to making a resource-less generic Execute Allow degrade to Ask for the reserved `background:` Command namespace, with explicit Deny precedence and focused tests; no public permission schema or PERM-006-D/E behavior. Windows/detached/unsupported shapes fail closed. Exclude TOOL-024-C/D, CLI/TUI/Dashboard/I213 production files, persistence, `/auto`, release and unrelated permission behavior. |
| Claimed At | 2026-08-23 |
| Source Issue | #59 |
| Governance Claim PR | #379 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer long-task objective selects Issue #59 and explicitly authorizes the bounded I213/I222-B parallel pair in Issue #366 comment `5386904546`. Claim PR #379 exact head `5f0816aa` passed CI `32650593056`, independent Agent-role claim review `5386970071` and merge-time CAS `5386973729`, then merged as `48e8ae9b`. Implementation still requires fresh exact-head process/permission/unsafe/API review. |
| Implementation PR | #382 — merged as `8671edf45c168612bfa4a4bbb65a9847026e1b96` |
| Last Updated | 2026-08-24 |
| Handoff / Release Condition | I222 is closed with implementation evidence. TOOL-024-C requires its own owner, runnable iteration and effective claim; Windows D1/D2 and I223 remain separate. |

## Published Baseline

Planning target: `main@e1c375e6` after I221/PERM-006-C closeout.

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| TOOL-024-B | TOOL-024 | Ready / Unclaimed | A/I188, TOOL-023-C, RUNTIME-005, PERM-006-C Complete; ADR-060 Accepted | Unix supervisor core with bounded receipt/output, exact background permission, whole-group cleanup and terminal event |

### Current Nonterminal Inventory And Disposition

| Iteration(s) | State | I222 disposition |
|---|---|---|
| I164 | Paused / superseded | Do not restore. |
| I197/I198/I201/I210 | Review / Claimed | Preserve corrective/deferred-validation owners; no scope transfer. |
| I206-I208 | Planned / Unclaimed | Preserve ordered steering sequence; do not activate. |
| I213 | Active / Claimed; PR #372 DIRTY against current main | Preserve Dashboard/CLI owner. I222 excludes its production files; overlap pauses I222. |
| I222 | Planned / Unclaimed in Draft claim | Proposed next Unix-only process-security slice; no authority before claim merge. |
| I223 | Planned / Unclaimed | Evidence-only cleanup for Issue #378; do not activate before implementation rows exist. |

PRs #120/#121 remain archival Drafts. PR #372 and the local I213 branch retain their existing
Dashboard/CLI work; their old head/base evidence cannot be reused after rebase. Shared governance
files use owner-first union updates only. I222 cannot become effective Active beside I213 unless the
maintainer explicitly authorizes this exact non-overlapping pair; Draft preparation is not that
authorization.

### Scope

- Implement the exact TOOL-024-B owner scope and ADR-060 B row.
- Production authority: `talos-agent`, `talos-tools`, `talos-core`, necessary additive
  `talos-runtime` finalizer integration, manifests if mechanically required, focused tests and
  directly affected API/runtime documentation.
- Record complete changed-file inventory before the first implementation push.

### Non-Goals

- No `crates/talos-cli/**`, `crates/talos-dashboard/**`, I213/WEB owner or Dashboard behavior.
- No process tool, Windows Job Object, CLI/TUI projection, persistence, PTY/stdin or unsupported
  multi-command background execution.
- No detached/self-daemonizing Unix command or guarantee for a child that deliberately escapes the
  Talos-created process group. Known detach shapes fail before permission/grant installation.
- No wall-clock terminal-job TTL. ADR-060's 32-terminal oldest-first cap plus session/process-end
  disposal is the documented retention policy.
- No `/auto`, PERM-006-D/E implementation, release/version/tag/publication or Desktop work.

### Acceptance

- The TOOL-024-B owner acceptance is satisfied without widening its exclusions.
- Unix leader/same-group-grandchild cleanup and ADR-060 unsafe boundary receive independent
  security review.
- Existing foreground shell/exec behavior remains compatible where observable.
- Windows and unsupported shapes fail before grant installation or spawn.
- Completion creates no provider request and no duplicate tool-result history.
- Deferred row V59-B1 is bound to the exact implementation head in Issue #378.

### Planned Validation

- Focused `talos-tools`, `talos-agent`, `talos-core` and `talos-runtime` locked tests.
- Unix subprocess integration test with child and grandchild cleanup assertions.
- Permission tests for exact `background:` separation and fail-closed paths.
- External `talos-runtime` consumer fixture for any additive public runtime seam.
- `./scripts/release_preflight.sh` and both governance validators.
- `git diff --check`, staged-diff/secret/EOF/generated-residual audit.
- Exact-head CI and independent process/permission/unsafe/API security review.

### Documentation To Update

- TOOL-024-B and I222 owners first; parent TOOL-024 and long-task checkpoint second.
- API/runtime documentation for the delivered B boundary.
- Board, backlog, iteration index and manifest only after owner truth changes.
- Issue #59 and deferred validation Issue #378 evidence comments.

### Risks And Rollback

- Direct-child kill mistaken for process-tree ownership: keep background admission disabled unless
  validated group ownership exists.
- Process-group cleanup mistaken for containment of a self-daemonizing process: reject known detach
  shapes and state the supported non-daemonizing contract in API/tool documentation.
- Foreground permission authorizes longer-lived work: require the exact separate background facet.
- Supervisor/event wiring overlaps I213: stop before editing an I213/CLI production file.
- Finalizer exceeds global deadline: report incomplete cleanup and never reset the deadline.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-23 | Atomic claim+activation candidate | PR #379 proposes Claimed/Active from `main@e1c375e6`; maintainer authorization `5386904546` permits only the exact non-overlapping I213/I222-B pair with stable-inventory/CAS gates. The proposal remains ineffective until merge. |
| 2026-08-24 | Claim+activation effective | PR #379 exact head `5f0816aa` passed docs-route CI `32650593056`, independent Agent-role claim review `5386970071` and merge-time CAS `5386973729`, then merged as `48e8ae9b`. I222 is Active/Claimed on `main`; implementation must start from this merge or later `main`, preserve the I213/I222-B pairwise boundary, and obtain fresh implementation-head process/permission/unsafe/API review. |

## Verification Evidence

- Local stable candidate was based on `main@7fd813e8322621b3ecc7c12c09a728c3762b0b67` (the #381
  merge); implementation PR #382 now carries the submitted candidate.
- `cargo check --offline --locked -p talos-core -p talos-permission -p talos-tools --features shell -p talos-agent -p talos-runtime --features shared-composition` passed.
- `cargo test --offline --locked -p talos-permission -p talos-agent -p talos-runtime` passed:
  permission 137, Agent 281 plus its integration fixtures, Runtime 38.
- `cargo test --offline --locked -p talos-tools --features shell process_boundary` passed, including
  Unix launcher output/reap and ESRCH cleanup tests.
- `cargo fmt --all` passed. Full workspace locked preflight, both governance validators, staged
  diff/secret audit and the exact changed-file inventory remain before first push.

## Local Stable Candidate Inventory

All current uncommitted production changes are within the effective I222 authority:

- `Cargo.lock` (required lockfile resolution for the existing workspace dependency graph)
- `crates/talos-core/src/background_job.rs`, `crates/talos-core/src/lib.rs`,
  `crates/talos-core/src/session.rs`, `crates/talos-core/src/tool/agent_tool.rs`
- `crates/talos-agent/src/background_jobs.rs`, `crates/talos-agent/src/lib.rs`,
  `crates/talos-agent/src/configuration.rs`, `crates/talos-agent/src/session.rs`,
  `crates/talos-agent/src/tool_execution.rs`
- `crates/talos-tools/src/process_boundary.rs`, `crates/talos-tools/src/bash_tool.rs`,
  `crates/talos-tools/src/exec_tool.rs`
- `crates/talos-permission/src/rule.rs`, `crates/talos-permission/src/lib.rs`,
  `crates/talos-permission/src/permission_tests.rs` (the #381-amended permission namespace)
- `crates/talos-runtime/src/lib.rs`, `crates/talos-runtime/src/shutdown.rs`

No `crates/talos-cli/**`, `crates/talos-dashboard/**`, README, I213/WEB owner, `/auto`, process
tool, Windows implementation, persistence or release file is in this candidate.

## Completion Evidence

- Implementation PR: #382 (merged as `8671edf45c168612bfa4a4bbb65a9847026e1b96`).
- Completion Commit: `8671edf45c168612bfa4a4bbb65a9847026e1b96` (pre-existing implementation merge).
- I223/Issue #378 remains a separate deferred validation cleanup and does not block this B owner
  closeout; it remains required for final Issue #59 closure.

## Variance And Residuals

- The 2026-08-24 permission-namespace checkpoint below records one necessary scope correction.
  C/D and Issue #378 remain explicitly separate.

## Permission Namespace Change-Control Checkpoint — 2026-08-24

Local design inspection after activation proved that a resource-less nature-based Execute Allow
currently matches every Command facet, including `background:<tool>:<resource>`. Merely adding the
ADR-060 background facet would therefore let a foreground/generic policy Allow authorize a
longer-lived process, violating the Accepted decision.

I222 adds only the minimum permission-engine authority required to close that gap:

- for a Command facet whose concrete resource starts with reserved `background:`, a resource-less
  generic Execute Allow cannot satisfy that facet and evaluation degrades to Ask;
- explicit Deny rules retain precedence, and an explicit matching `background:` resource rule or
  exact Session grant may authorize it;
- focused tests prove foreground generic Allow is insufficient and exact background approval does
  not widen to generic Execute;
- no permission config/schema/API change, typed-effect work, PERM-006-D/E behavior, `/auto`,
  Dashboard, CLI or persistence change is authorized.

For current execution, this checkpoint supersedes only the historical Production authority row in
the preserved Published Baseline. The added production inventory is limited to
`crates/talos-permission/src/rule.rs` and focused permission tests in
`crates/talos-permission/src/permission_tests.rs`; it does not reopen the rest of
`crates/talos-permission/**`.

No `talos-permission` production edit may start until this owner amendment reaches `main` with
exact-head CI, independent permission/security/API review and merge-time CAS. The already local,
uncommitted core/agent protocol sketch remains paused and is not implementation evidence.

## Retrospective

- Pending execution.
