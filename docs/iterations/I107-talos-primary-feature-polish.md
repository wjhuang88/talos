# Iteration I107: Talos-Primary Feature Polish

> Document status: Review
> Published plan date: 2026-07-08
> Planned objective: have Talos close the highest-priority issue-audit residual, then complete one
> low-risk user-facing feature or polish change as the primary development executor if capacity
> remains.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: a tested corrective reliability/UX change with Talos-primary implementation,
> validation, documentation, and REL-002 classification.

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| `SBT110` | 2026-07-08 self-bootstrap plan | Planned | I106 closeout | Highest-priority issue-audit residual selected and activated, starting with #18. |
| `SBT111` | 2026-07-08 self-bootstrap plan | Planned | SBT110 | Talos implements the selected corrective change through permission-gated tools. |
| `SBT112` | 2026-07-08 self-bootstrap plan | Planned | SBT111 | User docs, backlog, iteration, and board are synchronized owner-first. |
| `SBT113` | 2026-07-08 self-bootstrap plan | Planned | SBT112 | Session is classified against REL-002. |

### Scope

- Select from the issue-audit corrective queue before new polish:
  `RUNTIME-002`/`PROVIDER-002` #18 request-dispatch timeout, then `TUI-028` #39 dashboard transient
  notification, then #24/#31 visual evidence, then `TUI-029` #26 decision work.
- Select `TOOL-020` or the I085 MC107 walkthrough residual only after higher-priority corrective
  residuals are closed or explicitly blocked.
- Require real runtime evidence for any behavior change.
- Preserve permission, sandbox, credential, dependency, and storage boundaries.

### Non-Goals

- No new feature outside an existing owner doc.
- No broad refactor, dependency addition, permission-default change, or release action.
- No claim that external implementation qualifies for REL-002.

### Acceptance

- Given a corrective reliability or UX story is selected
  When Talos implements and validates it as primary executor
  Then the owner docs identify the changed behavior and runtime evidence.
- Given external review finds issues
  When remediation is needed
  Then Talos fixes them or the evidence is downgraded honestly.
- Given the session closes
  When REL-002 is updated
  Then it records whether this is a qualifying, partial, or non-qualifying feature/polish session.

### Planned Validation

- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- Real binary/runtime scenario for the selected story.
- `scripts/validate_project_governance.sh .`

### Documentation To Update

- Selected backlog owner doc.
- `docs/tasks/2026-07-08-four-month-talos-self-bootstrap-plan.md`
- `docs/backlog/active/REL-002-v1-self-bootstrap-release-gate.md`
- `docs/iterations/README.md`
- `docs/BOARD.md`

### Risks And Rollback

- Risk: the selected story turns out to require high-risk boundaries or external implementation.
- Rollback: defer that story, record why, and select a lower-risk owner with the same monthly goal.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-07-08 | Planning | Created as Month 2 of the 2026-07-08 Talos-primary self-bootstrap plan. |
| 2026-07-09 | Activation (SBT110) | I107 activated. Selected #18 request-dispatch timeout (RUNTIME-002 / PROVIDER-002) as the highest-priority corrective queue item. Runtime: glm-5.2 via zai-coding-plan (external, not `talos`). Smoke baseline: 9/9 passed at `2d925fe`. Per default decisions in the four-month plan, #18 outranks #39 dashboard notification, #24/#31 visual evidence, and TOOL-020/I085 polish. |
| 2026-07-09 | Implementation (SBT111) | Implementation of request-dispatch timeout for OpenAI-compatible and Anthropic providers in progress. |
| 2026-07-09 | Implementation (SBT111) | #18 request-dispatch timeout fixed. Added `dispatch_timeout_secs` to `ProviderTimeoutConfig` (default 60s). Wrapped `send().await` in `tokio::time::timeout` for both OpenAI and Anthropic providers. 4 provider tests added. Follow-up audit added 2 bridge tests proving dispatch timeout emits `AgentEvent::Error` and terminal `Status { is_processing=false, phase=TimedOut }`. Validation target: 1797 tests. REL-002: non-qualifying (external runtime). |
| 2026-07-09 | Docs (SBT112) | Owner docs updated in owner-first order: RUNTIME-002, PROVIDER-002, I107 (this doc), iteration README, BOARD, config reference. |
| 2026-07-09 | Closeout (SBT113) | Session classified non-qualifying for REL-002 (external runtime glm-5.2). I107 moved to Review. |

### SBT110 Selection Rationale

The four-month plan § Default Decisions states: "For I107, select #18 request-dispatch timeout before TOOL-020, I085 MC107, or TUI polish. A P0 stuck-processing residual outranks lower-risk feature polish." GitHub Issue #18 was incorrectly closed — per the 2026-07-08 Status Correction in RUNTIME-002 and PROVIDER-002, the root cause (provider HTTP request dispatch can hang before response headers arrive) was not fixed. The existing `ProviderTimeoutConfig` fields (`first_packet_timeout_secs`, `stream_idle_timeout_secs`) only protect stream parsing after a response exists, not the `send().await` phase before response headers arrive.

### Runtime Boundary Classification

This session is executed by glm-5.2 via zai-coding-plan (external runtime), not the `talos` binary. Per REL-002 acceptance criterion 7 and the four-month plan § Operating Rules, any code/doc edits performed by an external runtime are classified as **non-qualifying** evidence for REL-002. The artifacts produced may still be useful for future Talos-primary sessions, but this session does not prove self-bootstrap capability.

## Verification Evidence

- #18 request-dispatch timeout: fixed in SBT111. 4 provider tests plus 2 agent/CLI bridge tests
  cover dispatch timeout and terminal processing cleanup.
- Follow-up validation after bridge-test sync: `cargo fmt --all -- --check`,
  `cargo check --workspace`, `cargo test --workspace`,
  `cargo clippy --workspace -- -D warnings`, `scripts/validate_project_governance.sh .`,
  `git diff --check`, and `scripts/talos_smoke.sh` all passed.
- Validation matrix: cargo fmt, check, test, clippy, governance, git diff --check, talos_smoke.sh all green.
- REL-002 classification: NON-QUALIFYING (runtime was glm-5.2 external, not talos binary).

## Variance And Residuals

- The #18 fix is complete. The #39 dashboard transient notification and #24/#31 visual evidence remain open for selection in a future iteration if Talos-primary capacity remains.

## Retrospective

- This iteration's code changes were executed by external runtime (glm-5.2 via zai-coding-plan). Per REL-002 criterion 7, the session is non-qualifying. The technical fix (request-dispatch timeout) is correct and tested, but the self-bootstrap capability was not demonstrated.
