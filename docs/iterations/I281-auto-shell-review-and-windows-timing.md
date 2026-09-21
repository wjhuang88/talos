# Iteration I281: Model-First Shell Approval And Windows Lifecycle Tests

> Document status: Planned
> Planned objective: Model assessment for all Auto-mode shell Ask requests, and reliable Windows lifecycle tests.
> MVP deliverable: Runnable bash/PowerShell approval with actionable human decision points and real Windows process-tree cleanup acceptance.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | None |
| Executing Agent | None |
| Claimed At | Not applicable |
| Governance Claim PR | Not applicable |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer selected planning on 2026-09-21; no implementation activation. |
| Implementation PR | Not started |
| Last Updated | 2026-09-21 |
| Handoff / Release Condition | Decision reconciliation and effective claim required before implementation. |
| Source Issue | Maintainer request 2026-09-21; related #188, #563 and #234 CI residual |
| Work Slice | I281-A shell Ask review; I281-B Windows lifecycle test reliability. Planning only. |

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

- 2026-09-21 acceptance checkpoint: maintainer explicitly accepted ADR-081. The prior decision
  wait below is historical and resolved. Next step: atomic governance claim and activation;
  implementation remains unstarted until that record reaches main.

- Decision proposal: [ADR-081](../decisions/081-all-ask-shell-assessment.md) separates assessment
  of all Ask requests from automatic execution authority and preserves configured human Ask.
  Await maintainer acceptance before activation of changed policy.

- 2026-09-21: Maintainer requested both items for the next cycle. Recorded locally as
  Planned/Unclaimed; no implementation, new remote subtask Issues or PRs started.
