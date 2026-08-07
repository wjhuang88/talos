# Iteration I179: Core Tool Facade Decomposition

> Document status: Planned
> Published plan date: 2026-08-07
> Planned objective: decompose private result/presentation, authorization, tool-trait, contribution/registry, and protocol responsibilities from `talos-core/src/tool.rs` without changing public paths or behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `talos_core::tool` remains the stable public facade while its existing implementations have focused private source ownership, with downstream public-path probes and all core/workspace behavior checks passing.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Move existing result/presentation, authorization, `AgentTool`, contribution/registry, and protocol implementations from `talos-core/src/tool.rs` into private responsibility modules behind the unchanged public `talos_core::tool` facade; preserve every public path/name, visibility, trait default, object-safety property, serialization/schema shape, authorization normalization/comparison rule, registry replacement/collision/validation semantic, diagnostic, macro, dependency, and protocol parse/config behavior. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | Pending |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | Not started |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Release if exact public-path/API/serialization/trait/registry/authorization/protocol equivalence cannot be proven; any API redesign or semver change requires a separate story, ADR, and migration plan. |

Before activation, follow `docs/sop/AGENT-COLLABORATION.md`. The claim is ineffective until the
finalized `Claimed` record is merged into `main`.

## Non-Terminal Inventory At Selection

| Item | State | Disposition |
|---|---|---|
| I159-I162 | Blocked | Unchanged; their feature/composition/security/publication dependency chain remains blocked. |
| I164 | Paused | Historical superseded target; no activation. |
| Recovery PRs #120/#121 | Open archival evidence | Immutable; no implementation authority. |
| ARCH-034-R04 | Refinement | Native/panic/unsafe boundary remains excluded pending independent security review. |
| I178 / ARCH-034-R09 | Closed | Implementation merge `f9263480`; closeout merge `bd347939`; no overlap with core tool-facade source ownership. |
| ARCH-034-R11 | Ready / unclaimed | Retained for a later independent documentation-truth claim; no overlap with this source split. |

No other Active, Review, or Planned iteration overlaps this work. R10 is selected only after I178
closure and the current-state audit confirmed a private source-ownership issue behind an otherwise
correct, dependency-free public facade.

## Published Baseline

### Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| ARCH-034-R10 | ARCH-034 | Ready | I171 architecture register, I158 tool-composition semantics, I178 closure, and existing `talos-core`/workspace tests | One behavior-preserving private source split behind the stable `talos_core::tool` facade. |

### Scope

- Keep `talos_core::tool` as the sole stable public facade and preserve all existing public paths.
- Move existing result/presentation types, authorization types/helpers, `AgentTool`, contribution/registry logic, and protocol types into focused private modules.
- Preserve exact derives, attributes, documentation links, visibility, method bodies, trait defaults, object safety, normalization, validation, collision behavior, diagnostics, constants, and macro behavior.
- Add private source-layout checks and compile-time downstream probes covering the current public API surface.

### Non-Goals

- No public module/path/name, visibility, signature, trait-default, serialization/schema, registry, authorization, protocol, macro, dependency, or diagnostic change.
- No new abstraction, feature flag, crate, dependency, compatibility alias, or deprecation.
- No changes to R04, R11, I159-I162, permission policy, tool composition, or runtime execution behavior.

### Acceptance

- Existing internal and downstream imports through `talos_core::tool::*` compile unchanged.
- The facade exposes exactly the existing public types, trait, methods, macro behavior, and protocol configuration surface without exposing private implementation modules.
- Existing result/presentation, authorization, contribution/registry, serialization, protocol, and macro tests pass unchanged.
- Mechanical source-body/literal checks and focused probes show that the change is source movement only, with `talos-core` remaining dependency-free.

### Planned Validation

- `cargo test -p talos-core --locked --no-fail-fast`
- `cargo clippy -p talos-core --all-targets --locked -- -D warnings`
- `cargo fmt --all -- --check`
- `cargo check --workspace --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test --workspace --locked --no-fail-fast`
- `./scripts/release_preflight.sh`
- `scripts/validate_project_governance.sh .`
- `bash scripts/validate_collaboration_claims.sh .`
- `git diff --check`
- Mechanical source-body/string-literal equivalence checks and downstream public-path probes.
- Exact-head Unix/Windows CI and rebuilt CLI smoke.

### Documentation To Update

- Synchronize ARCH-034-R10, I179, `docs/iterations/README.md`, `docs/backlog/PRODUCT-BACKLOG.md`, `docs/BOARD.md`, and the governance manifest.
- No user-facing behavior documentation change is expected because this is a private source decomposition with stable public paths.

### Risks And Rollback

- Risk: re-export order or module privacy changes downstream resolution, derive/schema output, trait defaults, registry diagnostics, authorization path handling, or protocol behavior despite compiling locally.
- Rollback: revert the private module move if exact API/source/behavior equivalence cannot be shown; public redesign requires a separate semver-governed story, ADR, and migration plan.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-08-07 | Planning | I179 selected after inventorying non-terminal work, confirming I178/R09 closure, and finding no overlapping effective claim or implementation PR. |

## Verification Evidence

- Claim-only preflight and current `tool.rs` public/downstream surface inventory are recorded in the session; implementation evidence is intentionally absent until the claim becomes effective.

## Completion Evidence

- Completion Commit: not assigned; retain Planned until claim and implementation evidence exist.

## Variance And Residuals

- R04 remains Refinement pending independent security review.
- R11 remains separately owned and independently claimable after I179 closes.

## Retrospective

- Outcome: pending.
- Documentation: pending implementation result; no user-facing behavior documentation change is planned.
- Lessons: none recorded.
