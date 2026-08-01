# MODEL-011: Custom Model Capability Probe

| Field | Value |
|---|---|
| Story ID | MODEL-011 |
| Type | Product / TUI / Configuration Story |
| Priority | P2 |
| Status | Refinement — probe protocol, evidence precedence, cost UX and persistence schema require an iteration decision |
| Source | [GitHub Issue #124](https://github.com/wjhuang88/talos/issues/124) |
| Selected Iteration | None |
| Depends On | MODEL-008-A; MODEL-009-B; TUI-033; current provider adapters and atomic config mutation boundary |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #124 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-01 |
| Handoff / Release Condition | Refine the probe decision, evidence precedence and persistence migration; establish a claim before implementation. |

## Identity / Goal / Value

Provide an explicit, parameterless `/model-probe` workflow for configured custom models so a user can run a bounded synthetic capability suite, review evidence quality, and atomically persist fresh, secret-free probe evidence without enabling background probing or relying on model-name guesses.

## Scope

- Picker contains only configured custom models and carries structured provider/model identity.
- Confirmation explains provider cost and guarantees synthetic, workspace-independent inputs.
- Bounded probes independently evaluate text, streaming, native tool calls, continuation, structured output, image input, parallel calls and usage accounting.
- Results distinguish Supported, Unsupported, Degraded and Unknown; transport/auth/rate-limit failures never become Unsupported.
- Probe-owned evidence is atomically persisted through current config types, remains separate from explicit overrides and built-in catalog metadata, and becomes stale when fingerprint inputs change.
- Virtual probe tools are in-memory only and cannot reach the normal tool dispatcher.

## Exclusions

- No automatic startup/background probe, unbounded context-window search, real tool execution, workspace/session/memory input, second capability database, credential persistence, or silent override of user-authored metadata.

## Decision Links And Constraints

- Capability evidence that enables image/tool composition must remain fail-closed until precedence and freshness are accepted.
- Probe requests are billable network operations and require explicit confirmation.
- Config writes must use current atomic/concurrent mutation protections and abort if provider/model fingerprint changes during probing.

## Uncertainty And Validation Path

Refine a decision covering probe protocol/versioning, adapter-specific capability semantics, result precedence, fingerprint inputs, partial-run persistence, bounded retries, cancellation and secret-free diagnostics. Then select an isolated iteration and claim.

## State / Status Owners

- Story scope and acceptance: this file.
- Remote discussion: Issue #124.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## Acceptance For Future Implementation

- `/model-probe` opens a grouped configured-custom-model picker without a request.
- No provider call occurs before explicit cost confirmation.
- Native tool success requires a protocol-native, schema-valid virtual call; text imitation is not success.
- Synthetic image evidence verifies both protocol acceptance and content recognition.
- One final atomic write preserves unrelated config and secrets; cancellation writes nothing.
- Stale evidence remains diagnostic but cannot silently enable gated functionality.
- Full command/panel, probe-runner, persistence, integration, security and current-governance tests from Issue #124 pass.

## Residual Destination

Extended maximum-context probing, automated fleet probing and provider-specific benchmark suites require separate owners and cost/privacy decisions.
