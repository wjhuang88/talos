# Iteration I189: PERM-006-A Structured Permission Decisions

> Document status: Planned
> Published plan date: 2026-08-11
> Planned objective: add one behavior-preserving structured permission request, evaluation context and per-facet decision-report contract as the implementation source for existing permission entrypoints.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: locked permission and workspace tests prove one structured evaluator reproduces the existing Deny/Ask/Allow matrix while exposing redaction-safe per-facet provenance through additive compatibility APIs.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 implementation session 2026-08-11 |
| Work Slice | Implement only PERM-006-A / I189: add one structured permission request/context/per-facet decision-report evaluator, delegate existing permission entrypoints to it, preserve current Deny/Ask/Allow outcomes and compatibility-visible Deny messages, and add provenance, redaction, fail-closed and order-independence tests. No approval routing, wrapper removal, grant/store, AlwaysApprove, typed-resource, policy, sandbox, PERM-006-B/C/D/E, PERM-007, TOOL-024, ACP or release change. |
| Claimed At | 2026-08-11 |
| Source Issue | #53 |
| Governance Claim PR | #197 |
| Authorization Mode | Independent review |
| Authorization Evidence | Independent security review is mandatory on the finalized exact head before merge; no approval exists yet. This proposed claim remains ineffective until target-branch merge. |
| Implementation PR | Not started |
| Last Updated | 2026-08-11 |
| Handoff / Release Condition | Obtain independent exact-head security review, pass CI and merge-time CAS, and merge PR #197 before implementation; explicitly disposition current non-terminal iterations before activation. |

## Published Baseline

### Selected Stories

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| PERM-006-A | PERM-006 / Issues #52 and #53 | Refinement; additive compatibility design required | PERM-004/PERM-005 boundaries; first child in A→E order | One structured request/context/report evaluator with unchanged permission decisions and compatibility projections |

### Scope

- Add structured permission request, execution context, aggregate report, per-facet report, stable rule/grant provenance and redaction-safe reason types in the existing crate ownership boundary.
- Make one `evaluate_request`-style entrypoint the implementation source for current `evaluate`, `evaluate_with_nature` and `evaluate_profile` compatibility methods.
- Preserve conservative multi-facet aggregation independent of input order: any Deny, otherwise any Ask, otherwise Allow.
- Preserve configured policy, runtime grants, workspace trust, mode restrictions, default behavior and invalid-resource fail-closed precedence.
- Add behavior-characterization, compatibility, every-mode/precedence and aggregation property/table tests; update permission diagnostics/reference docs for the additive contract.
- Assess public API and hook DTO compatibility before editing; accept an ADR/migration plan first if review finds a breaking or serialized contract change.

### Non-Goals

- No approval routing, prompt/UI ownership, permission-aware wrapper removal, duplicate-engine convergence or execution-pipeline migration.
- No grant compiler/store, `AlwaysApprove` scope, typed effect/resource migration, policy broadening, sandbox change or persistent background-task permission.
- No correction to shipped permission decisions. Any discovered behavior defect becomes a separately reviewed security fix.
- No PERM-006-B/C/D/E, PERM-007, ACP, TOOL-024 or release implementation.

### Acceptance

- Given every current permission mode and precedence class, when compatibility entrypoints delegate to the structured evaluator, then their Allow/Ask/Deny outcomes and compatibility-visible Deny messages match the frozen current matrix.
- Given a hybrid multi-facet request in any facet order, when evaluated, then the aggregate remains Deny > Ask > Allow and reports the responsible per-facet reasons.
- Given an observer-facing report, when serialized, logged or rendered, then private projected fields, raw tool input and secrets are absent.
- Given a consequential facet requiring a concrete resource, when that resource is missing or invalid, then evaluation fails closed exactly as the current security contract requires.
- Given configured/default/runtime/workspace/mode decisions, when reported, then provenance is stable and testable without changing rule precedence or grant lifetime.
- Existing permission, agent, CLI, TUI, runtime, MCP, plugin and workspace tests remain green.

### Planned Validation

- `cargo fmt --all --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test -p talos-permission --locked`
- `cargo test -p talos-agent --locked`
- `cargo test -p talos-runtime --locked`
- `cargo test --workspace --locked`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- Exact-head independent security review of API compatibility, Deny precedence, redaction and fail-closed resource handling.

### Documentation To Update

- `docs/backlog/active/PERM-006-A-structured-permission-decisions.md`
- `docs/backlog/active/PERM-006-permission-pipeline-convergence.md`
- permission architecture/reference and public API migration notes actually affected by implementation
- `docs/iterations/README.md`, `docs/backlog/PRODUCT-BACKLOG.md` and `docs/BOARD.md`
- User-facing README remains unchanged unless the implementation adds a new observable diagnostic surface.

### Risks And Rollback

- Risk: refactoring evaluation can reorder Deny/Ask/Allow precedence while compatibility tests still cover only common paths.
- Risk: structured reasons can leak raw resources, private projection fields or secrets to hooks and display surfaces.
- Risk: public type placement can accidentally create a semver-breaking dependency direction.
- Rollback: retain existing compatibility entrypoints and revert the additive evaluator/types; no configuration or durable-data migration is permitted in this slice.

## Non-Terminal Coordination Record

- I185 remains Planned under its separate SQLite validator claim and PR #191.
- I186/TUI-046-B remains separately owned by its claim/implementation chain and PR #193.
- I187/SESSION-008-A remains Review in PR #195; I189 remains Planned until current non-terminal work receives an explicit activation disposition.
- I188/TOOL-024-A is only a separate proposed decision claim in draft PR #196; it does not authorize PERM-006 work and does not overlap this additive evaluator slice.
- I159-I162 remain Blocked under their existing gates; I164 remains Paused.
- Permission code is security-sensitive: independent review is mandatory before claim merge and implementation merge even when the intended decisions are unchanged.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-11 | Selection | PERM-006-A selected as a Planned additive foundation only. PR #197 proposes the claim but remains ineffective before merge; no implementation branch is authorized before independent review, validation and CAS. |

## Verification Evidence

- PR #197 records the finalized proposed claim; exact-head CI, both governance validators, independent security review and merge-time CAS gate merge.

## Completion Evidence

- No completion evidence. A status-only commit cannot certify this iteration.

## Variance And Residuals

- PERM-006-B/C/D/E remain blocked in their recorded order; the parent Epic and Issues #52/#53 remain open.
- Any behavior correction, grant-scope change or breaking API migration discovered during implementation must use a separate reviewed owner/change record.

## Retrospective

- Pending activation and execution.
