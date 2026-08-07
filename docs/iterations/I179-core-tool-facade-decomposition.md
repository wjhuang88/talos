# Iteration I179: Core Tool Facade Decomposition

> Document status: Complete
> Published plan date: 2026-08-07
> Planned objective: decompose private result/presentation, authorization, tool-trait, contribution/registry, and protocol responsibilities from `talos-core/src/tool.rs` without changing public paths or behavior.
> Baseline rule: once committed, preserve this target; changed targets use a new iteration ID.
> MVP deliverable: `talos_core::tool` remains the stable public facade while its existing implementations have focused private source ownership, with downstream public-path probes and all core/workspace behavior checks passing.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-5 architecture session 2026-08-07 |
| Work Slice | Move existing result/presentation, authorization, `AgentTool`, contribution/registry, and protocol implementations from `talos-core/src/tool.rs` into private responsibility modules behind the unchanged public `talos_core::tool` facade; preserve every public path/name, visibility, trait default, object-safety property, serialization/schema shape, authorization normalization/comparison rule, registry replacement/collision/validation semantic, diagnostic, macro, dependency, and protocol parse/config behavior. |
| Claimed At | 2026-08-07 |
| Source Issue | None |
| Governance Claim PR | #167 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | No independent reviewer is currently available; exact-head CI, both governance validators, merge-time CAS, and no blocking review feedback are required. |
| Implementation PR | #168 |
| Last Updated | 2026-08-07 |
| Handoff / Release Condition | Closed after implementation PR #168 merged; any public path/API/serialization/trait/registry/authorization/protocol, dependency, or behavior change requires a separate story, ADR, and migration plan where applicable. |

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
| 2026-08-07 | Claim submission | Draft governance claim PR #167 opened; the exact finalized `Claimed` record is submitted for claim-only CI and merge-time CAS. No implementation authority exists until #167 merges to `main`. |
| 2026-08-07 | Activation | Claim PR #167 finalized head `168d96b0ea9aedb9d3850c800f0cedddb09e76ef` passed exact-head CI `31183822345`; merge-time CAS confirmed no overlapping claim, implementation PR, or blocking feedback, and the claim merged as `9a5419e496db4f059ed841917d8ee9f099d377f6`. Implementation started from that effective claim. |
| 2026-08-07 | Review submission | The 1,731-line `tool.rs` was reduced to a 26-line stable facade over private result/presentation, authorization, `AgentTool`, registry, protocol, and test modules in source implementation commit `63d494c5`. Downstream public-path and private source-layout probes were added, and Draft implementation PR #168 was opened for exact-head CI and merge review. |
| 2026-08-07 | Completion | PR #168 squash-merged at `dafc9be08736aee91e0f9cdd92e5226930808061` from accepted exact Head `7b646a4d33cb17a21258e31d86ce1fe8d01b1929` after exact-head CI `31189425069`; merge-time CAS, both governance validators, remote owner reconciliation, installer fixture, rebuilt CLI smoke, and whitespace checks passed. |

## Verification Evidence

- Claim exact-head CI `31183822345` passed after a same-head rerun resolved a non-deterministic Windows timeout in three unrelated five-second async session tests; the accepted head did not change.
- `cargo test -p talos-core --locked --no-fail-fast`: passed the existing 62 core tests plus four I179 source-layout/public-path acceptance probes and doctests.
- `cargo clippy -p talos-core --all-targets --locked -- -D warnings`: passed.
- `cargo check --workspace --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, and `cargo test --workspace --locked --no-fail-fast`: passed.
- `./scripts/release_preflight.sh`: passed locked workspace format, check, Clippy, tests, doctests, governance, collaboration-claim, site, installer, and release gates.
- The sorted string-literal multiset, public struct/enum/trait names, public inherent method names, and `AgentTool` trait method names match effective claim merge `9a5419e4` across the facade plus private modules.
- Exact-head CI `31189425069` passed Unix/Windows workspace, both governance validators, remote owner reconciliation, installer fixture, and rebuilt CLI smoke checks.
- Merge-time CAS confirmed base `9a5419e496db4f059ed841917d8ee9f099d377f6`, head `7b646a4d33cb17a21258e31d86ce1fe8d01b1929`, no blocking reviews/comments, and no overlapping claim or implementation PR.

## Completion Evidence

- Completion Commit: `dafc9be08736aee91e0f9cdd92e5226930808061`

## Variance And Residuals

- R04 remains Refinement pending independent security review.
- R11 remains separately owned and independently claimable after I179 closes.
- This iteration is library-only private source organization. It changes no user-facing behavior, so binary runtime acceptance and user-facing documentation changes are not applicable; compile-time downstream probes and locked workspace behavior tests own acceptance.

## Retrospective

- Outcome: Complete; behavior-preserving private core tool-facade source decomposition delivered.
- Documentation: governance owners synchronized; no user-facing behavior documentation change was required.
- Lessons: none recorded.
