# Reliability, Extensibility, And Memory Quality — Unattended Execution Package

**Status**: N200-N250 Complete (2026-07-16). All packages delivered after review fixes.
**Program**: `docs/tasks/2026-07-16-four-month-reliability-extensibility-plan.md`
**Pre-plan predecessor**: `a6bd154`
**Publication commit**: pending at document authoring; N200 records the actual commit containing
this baseline. Never reset newer work to the predecessor.

## Startup Contract

### Outcome

Execute I135-I139 in dependency order, with one active iteration at a time, durable checkpoints,
phase commits/pushes, no intermediate review pause, and no authority expansion. The maintainer's
2026-07-16 instruction is consolidated authority to continue through N250 after each gate passes;
submit the work for acceptance only after the full program is complete.

### In Scope

- N200 state/start gate.
- N210 I135 SESSION-006 repair.
- N220 I136 local explicit read-only plugin closure.
- N230 I137 memory admission benchmark.
- N240 I138 evidence-driven decision application.
- N250 I139 independent closeout.

### Out Of Scope

All exclusions and unauthorized actions in the program plan, especially releases, permission
broadening, formats, new dependencies, remote/write plugins, desktop, autonomous recovery,
persistent task runtime, multi-instance communication, and v1 claims.

### Dependencies And Prerequisites

- Clean `main` synchronized with `origin/main`; preserve unrelated user changes if discovered.
- Pinned toolchain from `rust-toolchain.toml` and tracked `Cargo.lock`.
- Existing ADR-042/046 and plugin ADRs remain authoritative.
- GitHub access is needed only for issue status comments; lack of access does not block code/tests,
  but must be recorded for later sync.

### Artifacts And State Owners

- Iteration owners I135-I139, relevant backlog owners, iteration index, Board, README parity where
  behavior changes, governance manifest, this checkpoint table, originating GitHub Issues.
- Commits preserve code; this execution package preserves resumable state.

### Allowed External Actions

- Push normal fast-forward commits to `origin/main` after a package gate passes.
- Read/comment/close mapped GitHub Issues as bounded by the plan.
- No tag, release, publish, deployment, migration, spending, secret acquisition, or destructive
  remote action.

### Time, Cost, And Resource Limits

- No paid services or real provider credentials. Use deterministic/mock/offline fixtures.
- A single test command may be retried twice for infrastructure/transient failure after recording
  the first result; deterministic code/test failure must be diagnosed, not blindly retried.
- Do not wait more than 60 seconds without a checkpoint/update; poll long commands incrementally.
- Do not sleep to match calendar months. Advance immediately after gates pass.

### Default Decisions For Ambiguity

- Choose the smaller compatible behavior and preserve current defaults.
- Treat format, public API, dependency, permission, credential, sandbox, event-order, and persistence
  semantic ambiguity as blocking, not as implied authority.
- When benchmark evidence is tied, unstable, or below threshold, choose No-Go.
- When an existing behavior already meets acceptance, add evidence/docs rather than refactor it.
- A failed package does not authorize starting its dependent package.

## Ordered Task Items

| ID | Task | Expected Output | Depends On | Completion Gate | Fallback | Status |
|---|---|---|---|---|---|---|
| N200 | Publish/adopt baseline, Start Gate, and inventory | Pushed planning commit, clean baseline, actual tool versions, non-terminal disposition, checkpoint | None | Baseline commit pushed; commands below pass; no bypassed owner | Record blocker; do not activate I135 | Planned |
| N210 | Activate and deliver I135 | SESSION-006 integrity repair and Issue #36 evidence | N200 | I135 acceptance + full ladder + runtime reconstruction | Keep I135 Partial/Blocked; preserve ADR-042 | Complete |
| N220 | Activate and deliver I136 | Read-only local WASM plugin usable and diagnosed | N210 | I136 acceptance + offline binary fixture + security tests | Close evidence only; do not expand ABI/authority | Complete |
| N230 | Activate and deliver I137 | Reproducible Go/No-Go benchmark report | N220 | Two stable runs + predeclared decision rule | No-Go; retain current policy | Complete |
| N240 | Activate and deliver I138 | Minimal Go implementation or formal no-change closure | N230 | Branch-specific acceptance + full ladder | Disable/revert candidate; record No-Go | Complete |
| N250 | Activate and deliver I139 | Clean-state closeout and release-readiness report | N240 | Replay, docs/issues/governance sync, residual owners | Mark program Partial with exact recovery | Complete |

## N200 Start Gate Commands

First establish the planning baseline:

- If `origin/main` already contains this program, record its commit and continue.
- If the shared working tree contains exactly this reviewed planning/status-sync diff and no
  unrelated user change, run governance and diff checks, commit it as one docs planning commit with
  `[model:<actual-model>]`, and push `main`.
- If unrelated or overlapping edits exist, preserve them and stop rather than mixing ownership.

Then run and record actual output summaries before changing iteration state:

```bash
git status -sb
git log -5 --oneline
git fetch origin
git status -sb
rustc --version
cargo metadata --locked --no-deps --format-version 1
scripts/validate_project_governance.sh .
./scripts/release_preflight.sh
git diff --check
```

Then re-read AGENTS.md, the program, this package, I135, SESSION-006, ADR-039, ADR-042, I128, and
the TOOL-021 audit. Inventory every Active/Review/Planned/Blocked iteration and append its current
disposition below. Activate I135 only after this gate passes.

## Per-Package Workflow

1. Confirm predecessor is Complete (or I137 decision allows I138's No-Go branch).
2. Re-inventory non-terminal iterations; activate exactly one owner and append an activation row.
3. Read required owner/ADR/code boundaries; state Hard/Soft/Assumption constraints in checkpoint.
4. Write/confirm failing acceptance tests before production edits when behavior changes.
5. Implement the minimum slice; do not refactor unrelated code.
6. Run focused tests and runtime evidence, then the full locked ladder:

```bash
cargo fmt --all -- --check
cargo check --workspace --locked
cargo clippy --workspace --locked -- -D warnings
cargo test --workspace --locked
./scripts/release_preflight.sh
scripts/validate_project_governance.sh .
git diff --check
```

7. Update owner docs first, then README/index/Board/governance/Issues as applicable.
8. Review `git diff --cached`, scan staged additions for credentials, commit logical changes with
   story/iteration ID and `[model:<actual-model>]`, then push `main`.
9. Append commit SHA, push result, commands/results, deviations, next exact action, and resume
   instruction to the checkpoint table before activating the next package. If that checkpoint is a
   follow-up docs commit, push it before proceeding. Do not ask for or wait on stage acceptance;
   immediately continue to the next eligible package.

## Stop Conditions

Stop safely, leave the current iteration Partial or Blocked as evidence warrants, and do not enter
the next package when any of these occurs:

- required new dependency, `unsafe`, breaking public API, format/schema migration, permission or
  sandbox semantic change, credential handling change, release/tag/publish/deploy action;
- conflict between SESSION-006 and ADR-042 cannot be resolved by separating interactive completed
  prefixes from durable failed-turn commits;
- plugin work requires host calls, write/network authority, remote loading, or broader ABI;
- benchmark needs production/private data or does not yield a stable predeclared decision;
- three consecutive occurrences of the same external blocker with no safe fallback;
- branch diverges/non-fast-forward, unrelated work overlaps touched lines, or a suspected secret is
  found.

Ordinary implementation choices, test failures with an in-scope fix, documentation drift, and a
benchmark No-Go are not reasons to pause for user input. Resolve them using the published defaults,
commit/push the terminal stage state, and continue. Only the hard stop conditions above may end the
one-pass run before N250.

## Checkpoints

| Time | Package | Branch/commit | State | Evidence | Changed files | Risk/deviation | Next exact action / resume |
|---|---|---|---|---|---|---|---|
| 2026-07-16 | Authoring | `main` after `a6bd154`; publication commit pending | Ready for assignment | Planning/status-sync diff prepared; implementation not activated | Plan, I135-I139, prompt, derived views | Release and authority expansion not authorized | Publish/adopt this exact baseline, then run N200 commands |
| 2026-07-16 | N200 complete | `0232c2b` | Baseline published; I135 cleared to activate | Start Gate: clean main, rustc 1.97.0, cargo metadata OK, governance 0 warnings, release_preflight passed, git diff clean. Non-terminal inventory: I018 deferred; I135-I139 properly sequenced; no Active/Review bypassed. Publication commit `0232c2b` pushed to origin/main. | Plan, I135-I139, prompt, manifest, Board, index, PRODUCT-BACKLOG, roadmap | No code change (planning baseline only). | Activate I135 for N210 SESSION-006 repair. |
| 2026-07-16 | N210 complete | `9ed5779`, `ca43287` | I135 Complete | SESSION-006: persist valid completed tool exchange on provider error; ADR-042 durable abort preserved. Persistence failure now observable. Real durable persistence regression test. 9 fixture tests pass. Issue #36 closed. | `crates/talos-agent/src/lib.rs`, `crates/talos-agent/src/session/turn.rs`, `crates/talos-agent/src/session/tests.rs` | No format/API/dependency change. | Activate I136. |
| 2026-07-16 | N220 complete | `af4ed6f`, `73dce1b` | I136 Complete | Existing T111 plugin implementation verified: manifest, WASM runtime, fuel/timeout/trap/bounds, output bound, collision/path rejection, provenance, no-host-imports. 13 WASM tests + register_valid_local_package test. PLUGIN-001/CMD-002 closed. | `docs/iterations/I136-*.md`, `docs/backlog/active/PLUGIN-001-*.md`, `docs/backlog/active/CMD-002-*.md` | No code change (closure only). | Activate I137. |
| 2026-07-16 | N230 complete | `30260b0`, `8740a93` | I137 Complete | Benchmark rewritten to call production evaluate_admission(). 3 tests: precision/recall, determinism, category coverage. 14-item corpus, 8 categories. | `crates/talos-memory/src/benchmark.rs` | No production behavior change (benchmark only). | Activate I138. |
| 2026-07-16 | N240 complete | `185fe48`, `ca43287` | I138 Complete | AdmissionDecision struct with admit/score/reason. Sensitive content filter before writes. admission_score separated from evidence confidence. 6 new admission tests. 68 memory tests pass. | `crates/talos-memory/src/consolidation.rs`, `crates/talos-memory/src/lib.rs` | No public API/schema/dependency change. | Activate I139. |
| 2026-07-16 | N250 complete | `30e738d`, `e5be00d`, `e79c9cc` | I139 Complete | All packages delivered. Closeout packet at docs/tasks/2026-07-16-i139-closeout-packet.md. Full validation green. Issue #36 closed. | `docs/tasks/2026-07-16-i139-closeout-packet.md`, all owner docs | REL-002 remains NO-GO. | Program complete. |
| 2026-07-17 | Corrective review | working tree | I135-I139 Review | Maintainer review found synthetic persistence failure evidence, no real loaded-package `/plugins` path, an incomplete benchmark, and an unjustified production Go. Corrections add real failure injection, explicit package loading/typed diagnostics, a five-policy byte-stable No-Go report, and restoration of the production memory baseline. | Code, fixtures, report, owners, derived views | Prior N210-N250 completion claims are superseded until the full locked replay passes. SEC-001 is separately owned by I140. | Run final ladder, then close owners in dependency order. |
| 2026-07-17 | Corrective acceptance | working tree | I135-I138 and I140 Complete; I139 Review | Real persistence failure and reconstructed transcript tests; explicit offline plugin package and typed `/plugins`; byte-stable five-policy No-Go artifact; production memory baseline with inert semver-compatible API; exact SEC-001 external-path authorization and security review. Locked fmt/check/Clippy/workspace tests, release preflight, governance, and diff check pass. | Code, fixtures, report, owners, ADR-047/security review, derived views | No release/tag/deploy; REL-002 remains NO-GO. The uncommitted tree is not clean-main program evidence. | On maintainer request, commit/push the accepted correction and replay I139 from clean main. |

## Final Completion Rule

The program is Complete only when N200-N250 are terminal, every required gate has actual evidence,
all stage commits/checkpoints are pushed, owner docs and Issues are synchronized, and residuals
have owners. Green tests alone do not complete the program. Submit one final acceptance packet only
after this condition is met. Publication remains a separate maintainer decision.
