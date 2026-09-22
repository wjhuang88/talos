# Iteration I281: Model-First Shell Approval And Windows Lifecycle Tests

> Document status: Review
> Planned objective: Model assessment for all Auto-mode shell Ask requests, and reliable Windows lifecycle tests.
> MVP deliverable: Runnable bash/PowerShell approval with actionable human decision points and real Windows process-tree cleanup acceptance.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 |
| Claimed At | 2026-09-21 |
| Governance Claim PR | #587 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer requested I281 development and closure and accepted ADR-081 on 2026-09-21; claim #587 effective at cb00524d. Independent Agent security/API approval for #588 recorded in comment 5762107014. |
| Implementation PR | #588 (merged as a1cb869b), #591 (merged as bfdf8b67) |
| Last Updated | 2026-09-22 |
| Handoff / Release Condition | Implementation and corrective follow-up merged; filesystem no-effect evidence for timeout/Esc cancellation is still required before Complete. No release authorization. MODEL-007 remains unactivated. |
| Source Issue | Maintainer request 2026-09-21; related #188, #563 and #234 CI residual |
| Work Slice | I281-A bash/PowerShell Ask model assessment, script context and bound admission, actionable human escalation; I281-B Windows lifecycle test reliability. No release, Desktop or unrelated higher-risk-write roadmap. |

## Published Baseline

### Inventory And Selection

- Main baseline: `be7231dfbbb91c6d2ea1a8d0ff1dfc2e0461e29f`; I280/#234 Complete/Closed.
- I277 remains Review with human/device acceptance Deferred; I249 Planned/unselected;
  I164 Paused/superseded. No other current Active or Blocked iteration was found in the
  iteration document-status inventory. Their existing scope and evidence remain unchanged.
- Maintainer selects this combined cycle next. MODEL-007 catalog coverage follows this cycle,
  remains Refinement/Unclaimed and is deferred, not cancelled.
- Related owners PERM-007-E/F/G retain their existing scopes; this does not reopen completed
  slices or activate all higher-risk writes tracked by #563.

### I281-A: All-Ask Shell Model Assessment

- Auto enabled: every bash/PowerShell request requiring approval reaches model assessment,
  including compound commands, pipelines and scripts. Complexity alone cannot exclude review.
  Explicit Deny remains authoritative; valid existing authorization avoids redundant assessment.
  Auto disabled retains human approval.
- Supply actual command/script content, arguments, workspace context, relevant user intent and
  effect evidence. Resolve referenced scripts and relevant nested/source dependencies where
  feasible; explicitly identify unresolved dynamic or remote code.
- Do not upload full environments or secrets. Redacted, missing, oversized or incomplete evidence
  must be represented honestly and must never be treated as proof of safety.
- Reuse ADR-075 isolated bounded invocation. Treat script text as untrusted data, not instructions;
  do not inherit hidden tools or authority or recursively enter permission requests.
- Safe intent-aligned requests can receive exact-call AllowOnce only, never permanent broad
  grants. Bind review to actual execution inputs, script/dependency content, permission revision
  and admission. Changed content invalidates approval; hashing alone is not execution binding.
- HumanRequired explains concrete effects, unresolved risks and the choice required. Model errors,
  cancellation and timeout explicitly report incomplete assessment, not a fabricated risk verdict.
- Unknown execution identity requires re-review or safe refusal; a human click cannot bypass hard
  policy or sandbox constraints. Preserve exactly-once execution and all existing security gates.

### I281-B: Windows Timing Repair

- Own the I280 residual from CI `35551716705`, Windows job `106187581654`:
  `windows_agent_supervisor_timeout_cleans_the_job_tree` failed waiting for its grandchild marker.
  Retry success does not prove repair.
- Inspect `crates/talos-agent/tests/i226_windows_background_lifecycle.rs` and the supervisor.
  The 12-second execution deadline and 25-second marker wait permit termination before readiness;
  this is a structural risk, not proof of the particular failed process startup cause.
- Separate readiness synchronization from intentional timeout verification using an appropriate
  test clock/handshake design. Do not change production timeout semantics to make a test pass.
- Retain real Windows Job Object descendant cleanup, cancel and shutdown coverage. Assert terminal
  reason and cleanup; distinguish startup failure, early exit and marker failure in diagnostics.
- Add deterministic slow-start coverage and bounded cleanup on failure. No ignored tests, blanket
  retries or merely larger timeouts. Real Windows validation must run on GitHub Actions.

### Order And Gates

1. Establish an effective claim/activation before implementation; this plan is not activation.
2. Reconcile ADR-069/070 and W0/W1/F/G limits with the all-Ask requirement, record an amendment or
   new ADR and obtain required acceptance before changing policy. Requirement selection does not
   silently authorize higher-risk automatic writes or override accepted security contracts.
3. Repair I281-B alongside I281-A design, then locally converge implementation, adversarial tests,
   human escalation UX and documentation. Submit one combined stable implementation candidate.
4. Require fresh exact-head CI and independent Agent permission/security/API review before merge.
   Local fixes do not each need separate remote Issues or PRs.

### Acceptance And Validation

- Both shells: simple reads, complex read-only scripts, bounded writes, risky/unknown effects,
  Deny precedence, valid grants, Auto off, provider failure, malformed output, cancellation,
  stale revisions and scripts/dependencies changed between assessment and execution.
- Assert required content reaches the assessor; secrets and prompt injection cannot gain authority;
  missing evidence cannot produce unjustified AllowOnce. Safe requests execute exactly once without
  a panel; human escalation names decision points and failed assessment is visibly distinguished.
- Human UX walkthrough for explanations/modes; deferred checks remain explicitly unverified.
- Deterministic slow-readiness tests and real Windows focused/full workspace CI with descendant
  cleanup assertions. No claim of reliability based only on retry success.
- Pinned toolchain, locked focused tests and full release_preflight for implementation; both
  governance validators and diff check. Planning-only docs do not trigger local Rust builds.

### Documentation And Exclusions

- Update README.md, README.zh-CN.md, Auto permission guidance, relevant ADRs, test documentation
  and this acceptance ledger. Preserve historical checkpoints and append execution evidence.
- No release, Desktop work, permanent model-created trust, sandbox relaxation, unrelated dependency
  upgrades or wholesale #563 implementation. Residuals remain in this owner until explicit handoff.

## Planning Record

- 2026-09-22 closeout: #591 merged as `bfdf8b67f666602d3d39e2c5949adb646f46e01e`.
  Exact head `1fc3afd3fa9e1f3ec3ed9eb75017287b77117b9b`, base
  `368e4fb403940ee6858e00db7f2395ed2b9855e1`; CI `35676188615` passed all five
  applicable code/platform jobs, including full preflight and Windows workspace.
  Independent Agent security/API APPROVE and identity limits: comment `5770036176`.
  CAS confirmed unchanged head/base, no other open PR, and MERGEABLE; exact-head guarded
  merge succeeded. Global Issue reconciliation failed only on new #590 and was advisory
  under ADR-071; this closeout registers its separate AUTO-UX-001 owner and matrix row.
  Local corrective validation: formatting, TUI 592 tests, agent 408 tests and diff check passed.
  Human observations below partially cover the UX walkthrough; automated permission tests separately
  cover fail-closed decisions. Filesystem no-effect evidence for timeout/Esc remains outstanding, so
  this owner stays Review/Claimed.
  Locale UX remains #590; original-provider dispatch root cause is not proven by changing
  models or by diagnostic categories and remains an explicit residual in this owner.

- 2026-09-21 implementation acceptance checkpoint: #588 merged as
  `a1cb869bde965abc5676121362def594e3c20927`, exact head
  `8abf342d0e717dabee851145b9c23e461a105974`, base
  `cb00524dd7f4dd7698a47d08670457f76f372ed1`. CI `35609119644` passed all six
  jobs, including full release preflight and Windows workspace tests, PowerShell focused tests,
  direct walkthrough, governance validators and CLI smoke. Independent Agent security/API
  APPROVE is recorded in PR comment `5762107014`; this is Agent-role separation, not separate
  human identity. Merge-time checks confirmed unchanged head/base, CLEAN/MERGEABLE, no other
  open PR and no blocking feedback; merge used the exact-head match guard. Earlier Windows
  bool-byte compilation and nested-if Clippy failures were corrected before this passing head.
- Current state: Review/Claimed, implementation delivered; no Complete claim and no release.
  This checkpoint supersedes historical proposed/unstarted descriptions without rewriting them.
- Remaining human UX acceptance (not yet verified): with Auto on, an intent-aligned read-only
  shell request shows the model assessment and executes once; an uncertain/script request shows
  actionable effects and decision points before human choice; cancelling that choice performs no
  tool effect. With Auto off, permission requests still use human approval. Record observed UI
  outcomes and tested build here before final closeout. Do not use automated tests as evidence
  that a natural person completed this walkthrough. MODEL-007 remains unactivated.

- 2026-09-22 human UX acceptance completed on the local build: Auto-on bounded read-only
  compound command showed `model allowed once` and executed exactly once; Auto-off preserved the
  human approval path; an out-of-workspace write produced `model requested human approval` with
  concrete effects and decision points; approval timeout and explicit `Esc` cancellation both
  displayed denial and restored input (filesystem effects were not separately inspected);
  an explicit later authorization created a fresh review
  but still required the final permission approval. The walkthrough also confirmed that pending
  tool arguments no longer flash a duplicate JSON projection during model assessment. These are
  natural-person UI observations, not automated-test evidence.

- 2026-09-21 correction to the local checkpoint below: the implementation is not a stable
  candidate. Independent local review found missing script-evidence admission checks, an unsafe
  unknown-cwd compatibility path, and a public-context struct-literal compatibility break.
  These are being corrected locally. Remaining work includes all-Ask assessment coverage,
  actionable human escalation, full acceptance tests, user/API documentation and Windows CI.
  Workspace testing stopped at the `talos-skill` fixture attempting to create
  `~/.agents/skills/dedup-test`; tests after that failure were not proven by that run.
  Standard preflight subsequently failed during linking with `No space left on device`.
  Cargo build artifacts (18.5GiB) were cleaned; source changes remain local and uncommitted.

- 2026-09-21 local convergence checkpoint: authoritative normalized execution input is now passed
  separately from the redacted approval projection; registered shell tools provide their actual
  execution directory; capability-confined bounded script evidence is included in the contextual
  assessor payload without claiming execution-byte binding. Windows readiness tests align frozen
  Tokio time with real `std::time::Instant` deadlines. Focused Auto resolver tests (37) and locked
  workspace check pass. Full workspace tests are otherwise passing except the pre-existing
  `talos-skill` permission fixture failure (`Operation not permitted`) in the restricted host.

- 2026-09-21 actual activation: #587 merged as
  `cb00524dd7f4dd7698a47d08670457f76f372ed1`, exact head
  `7290815f6d924d7abc6f81c999036e6991810ce6`, base
  `be7231dfbbb91c6d2ea1a8d0ff1dfc2e0461e29f`. CI `35566243501` passed applicable
  jobs; independent security/API governance review `5756018598`, CAS `5756035516`.
  Implementation branch starts at that merge. Claim is now effective.

- 2026-09-21: #587 proposes atomic Claimed/Active status. This record has no target-branch
  authority until merge. Start implementation from that merge or later; preserve this proposal
  as historical evidence and append the actual activation checkpoint.

- 2026-09-21 acceptance checkpoint: maintainer explicitly accepted ADR-081. The prior decision
  wait below is historical and resolved. Next step: atomic governance claim and activation;
  implementation remains unstarted until that record reaches main.

- Decision proposal: [ADR-081](../decisions/081-all-ask-shell-assessment.md) separates assessment
  of all Ask requests from automatic execution authority and preserves configured human Ask.
  Await maintainer acceptance before activation of changed policy.

- 2026-09-21: Maintainer requested both items for the next cycle. Recorded locally as
  Planned/Unclaimed; no implementation, new remote subtask Issues or PRs started.

- 2026-09-21 local validation update: `cargo check --locked -p talos-agent`, `cargo test
  --locked -p talos-agent --lib` (404/404), `git diff --check`, both governance validators,
  the full workspace test phase, and the ordinary runtime SDK fixture passed. The coding-feature
  fixture reached external `aws-lc-sys` compilation but the host ran out of disk space while
  clang created temporary files; this is an environment limitation, not a Rust assertion failure.
  The fixture lockfile was synchronized and the 22.4GiB Cargo target was cleaned afterward.
  Windows-native lifecycle evidence remains pending GitHub Actions validation.
