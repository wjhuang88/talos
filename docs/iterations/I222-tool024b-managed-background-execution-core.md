# Iteration I222: TOOL-024-B Managed Background Execution Core

> Document status: Active / Claimed proposed by PR #379; ineffective until target-branch merge
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
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 Sol mainline Issue #59 session 2026-08-23 |
| Work Slice | Implement only TOOL-024-B under Accepted ADR-060: Unix Agent/session-owned supervisor, semantic pre-admission, exact background permission, bounded receipt/output/state/event, checked same-process-group cancellation/reap, and ordinary Session plus Runtime finalizer cleanup. Windows/detached/unsupported shapes fail closed. Exclude TOOL-024-C/D, CLI/TUI/Dashboard/I213 production files, persistence, `/auto`, release and unrelated permission behavior. |
| Claimed At | 2026-08-23 |
| Source Issue | #59 |
| Governance Claim PR | #379 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer long-task objective selects Issue #59 and explicitly authorizes the bounded I213/I222-B parallel pair in Issue #366 comment `5386904546`. Exact-head CI, independent process/permission/unsafe/API review and merge-time CAS remain required. |
| Implementation PR | Not started |
| Last Updated | 2026-08-23 |
| Handoff / Release Condition | Claim/activation is ineffective until #379 reaches `main`; implementation starts from that merge or later main. |

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

## Verification Evidence

- Pending implementation after effective claim.

## Completion Evidence

- Implementation PR: Not started.
- Completion Commit: Pending.
- A status-only commit cannot self-certify implementation; Deferred Human Validation keeps I222 in
  Review until I223 resolves V59-B1.

## Variance And Residuals

- None at planning time. C/D and Issue #378 remain explicitly separate.

## Retrospective

- Pending execution.
