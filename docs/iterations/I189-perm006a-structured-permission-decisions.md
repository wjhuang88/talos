# Iteration I189: PERM-006-A Structured Permission Decisions

> Document status: Review
> Published plan date: 2026-08-11
> Planned objective: add one behavior-preserving structured permission request, evaluation context and per-facet decision-report contract as the implementation source for existing permission entrypoints.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: locked permission and workspace tests prove one structured evaluator reproduces the existing Deny/Ask/Allow matrix while exposing redaction-safe per-facet provenance through additive compatibility APIs.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5.6 mainline implementation session 2026-08-21 |
| Work Slice | Implement only PERM-006-A / I189: add one structured permission request/context/per-facet decision-report evaluator, delegate existing permission entrypoints to it, preserve current Deny/Ask/Allow outcomes and compatibility-visible Deny messages, and add provenance, redaction, fail-closed and order-independence tests. No approval routing, wrapper removal, grant/store, AlwaysApprove, typed-resource, policy, sandbox, PERM-006-B/C/D/E, PERM-007, TOOL-024, ACP or release change. |
| Claimed At | 2026-08-11 |
| Source Issue | #53 |
| Governance Claim PR | #197 |
| Authorization Mode | Independent review |
| Authorization Evidence | Claim PR #197 merged as `0df88638409027849e5bf4ba13ef72d2e96b9b90` after exact-head CI `31554958547`, independent security approval comment `5261239200` bound to `b4f23ec2255c60723c7d1abae3084a24c3bb5899`, and merge-time CAS. Activation PR #351 merged as `20cfcce4e72be3da4e3efc1190ee498975e7476b` after exact-head CI `32500829272`, independent Agent-role security/governance approval `5372336921`, and merge-time CAS. |
| Implementation PR | #356 |
| Last Updated | 2026-08-22 |
| Handoff / Release Condition | Review implementation commit `6b577d6afcb05230c821214902b9067c45c767a9` through PR #356 with fresh exact-head CI, independent security/code review and merge-time CAS. A later owner-first closeout may cite that pre-existing implementation commit; this Review state does not authorize PERM-006-B/C/D/E or PERM-007 behavior. |

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

The following bullets are the preserved selection-time snapshot from 2026-08-11. The dated
activation table below is the current execution record and supersedes their old current-state
wording without rewriting it.

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
| 2026-08-21 | Claim reconciliation | PR #197 is merged as `0df88638`; its claim is effective. The earlier pending/ineffective wording is historical and is superseded by this execution fact. |
| 2026-08-21 | Non-terminal inventory | I197, I198, I201 and I210 remain Review under their separate corrective owners; I206-I208 remain Planned/Unclaimed; I213 remains Planned/Claimed and unactivated in the independent Dashboard lane; I164 remains Paused/superseded. None overlaps the PERM-006-A evaluator slice. |
| 2026-08-21 | Activation proposal | I189 is the only proposed Active iteration. The proposal changes governance state only and is ineffective until its exact head passes CI, independent security review and merge-time CAS and reaches `main`; no implementation branch or Rust/Cargo edit is authorized before then. |
| 2026-08-22 | Activation PR | Governance-only activation PR #351 is the proposed record. Its open branch has no activation effect; only the reviewed exact head reaching `main` authorizes implementation. |
| 2026-08-22 | Activation effective | PR #351 merged as `20cfcce4` after exact-head CI `32500829272`, independent Agent-role security/governance approval `5372336921` and merge-time CAS. I189 is Active/Claimed. |
| 2026-08-22 | API blocker | Read-only assessment proved that configured rules and runtime grants are indistinguishable in the public mutable `PermissionEngine.rules` vector. ADR-065 decision content commit `dae98460` records the required pre-1.0 encapsulation and migration boundary; no provenance guess or hidden sentinel is permitted. Its Accepted status is ineffective until exact-head review, CI, CAS and target-branch merge. |
| 2026-08-22 | ADR prerequisite complete | ADR-065 was Accepted through PR #355 merge `9579df7a` after exact-head CI `32508015164`, independent Agent-role security/API review `5373150265` and merge-time CAS. |
| 2026-08-22 | Implementation candidate | Commit `6b577d6afcb05230c821214902b9067c45c767a9` implements the bounded structured evaluator and moves I189 to Review through PR #356. It does not mark the iteration Complete or authorize a later child. |

## Verification Evidence

- PR #197 claim evidence: exact head `b4f23ec2255c60723c7d1abae3084a24c3bb5899`, CI `31554958547`, independent security approval comment `5261239200`, merge `0df88638409027849e5bf4ba13ef72d2e96b9b90`.
- Activation PR #351 evidence: exact head `c025f7b94cf71fb12650f24ad8f1fe1d2467f7bf`, CI `32500829272`, independent Agent-role security/governance approval `5372336921`, merge `20cfcce4e72be3da4e3efc1190ee498975e7476b`.
- ADR-065 decision content commit `dae98460c29c72cb61da391ddf998630e67d6f15` pre-exists its status commit. Acceptance remains ineffective until exact-head independent security/API review, CI, CAS and target-branch merge.
- ADR-065 acceptance superseded that pending checkpoint through PR #355 merge `9579df7a`, exact-head CI `32508015164` and independent Agent-role review `5373150265`.
- Local implementation evidence for `6b577d6a`: `cargo fmt --all --check`, locked workspace check,
  locked permission tests, permission Clippy with warnings denied, and locked dependent
  agent/runtime/CLI/MCP/plugin tests passed. Independent local red-team review approved after the
  closed reason dimension and MCP/plugin redaction coverage were added.
- A pre-correction full preflight run was intentionally interrupted after the red-team identified
  missing reason/redaction coverage; it is not claimed as final evidence. After correction,
  `./scripts/release_preflight.sh` completed successfully across the full workspace. Exact-head CI
  and independent implementation review remain pending.

## Completion Evidence

- No completion evidence. A status-only commit cannot certify this iteration.

## Variance And Residuals

- PERM-006-B/C/D/E remain blocked in their recorded order; the parent Epic and Issues #52/#53 remain open.
- Any behavior correction, grant-scope change or breaking API migration discovered during implementation must use a separate reviewed owner/change record.

## Retrospective

- Implementation is in Review; exact-head remote evidence and owner-first closeout remain pending.
