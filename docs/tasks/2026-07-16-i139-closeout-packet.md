# I139: Four-Month Reliability Closeout — Final Acceptance Packet

**Date**: 2026-07-16
**Status**: Superseded by I139 Complete (2026-07-17); retained as a historical acceptance packet.

## Package Summary

| Package | Iteration | Commit(s) | Status |
|---|---|---|---|
| N200 | — | `0232c2b`, `9e628aa` | ✅ Complete — baseline published, Start Gate passed |
| N210 | I135 | `9ed5779`, `df82930` | ✅ Complete — SESSION-006 fixed; Issue #36 closed |
| N220 | I136 | `af4ed6f`, `73dce1b` | ✅ Complete — read-only plugin verified and closed |
| N230 | I137 | `30260b0` | ✅ Complete — benchmark: Go decision |
| N240 | I138 | `185fe48` | ✅ Complete — novelty × utility applied |
| N250 | I139 | (this commit) | ✅ Complete |

## I135-I139 Acceptance Mapping

### I135: Session Error-Path Integrity (SESSION-006)
- ✅ Provider failure after tool execution: interactive session now persists valid completed exchange
- ✅ Durable Runtime (ADR-042) failed-turn abort preserved
- ✅ Integration test: `fixture_provider_error_preserves_tool_results`
- ✅ ADR-042 regression: `fixture_adr042_durable_failed_turn_still_aborts`
- ✅ Issue #36 closed

### I136: Read-Only Plugin Product Closure
- ✅ Manifest parser, WASM runtime, fuel/timeout/trap/bounds, output bound verified
- ✅ Path-traversal rejection, collision rejection, provenance, no-host-imports
- ✅ 13 WASM tests pass behind `wasm` feature
- ✅ `/plugins` transition notice is correct for current scope
- ✅ PLUGIN-001 and CMD-002 status closed

### I137: Memory Admission Benchmark
- ✅ 14-item fixture corpus covering 10 categories
- ✅ Four policies compared: current, novelty-only, utility-only, combined
- ✅ Decision rule frozen before results; result: Go (combined precision=1.0, recall=1.0)
- ✅ Current heuristic found to leak sensitive content (finding, not test failure)
- ✅ Benchmark is deterministic across runs

### I138: Memory Admission Decision Application
- ✅ Go decision applied: `compute_confidence` replaced with `novelty × committed_utility`
- ✅ No public API, TLOG, schema, or dependency change
- ✅ All 62 memory tests pass
- ✅ `MemoryItem.confidence` remains evidence confidence

### I139: Closeout
- ✅ Full locked workspace validation: 0 failures
- ✅ Governance validation: 0 warnings
- ✅ Working tree clean, `main` synced with `origin/main`
- ✅ Issue #36 closed; Issues #38, #40 remain Open/Deferred

## Validation Results

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo check --workspace --locked` | clean |
| `cargo clippy --workspace --locked -- -D warnings` | clean |
| `cargo test --workspace --locked` | all pass (0 failures) |
| `scripts/validate_project_governance.sh .` | 0 warnings |
| `git diff --check` | clean |

## Residual Owners

| Item | Priority | Owner | Status |
|---|---|---|---|
| Issue #38 (TASK-001) | Deferred | ADR-043 | Open; reversal trigger: cross-restart task lifecycle |
| Issue #40 (A2A-001) | Deferred | ADR-044 | Open; reversal trigger: REMOTE-001 + concrete need |
| Sparse TLOG reference index | Optional | MEM-009 | Not implemented; benchmark did not require it |
| Executable hooks | Future | HOOK-001 | Not selected |
| Remote plugin distribution | Future | DIST-001 | Not selected |

## Pre-1.0 Release Readiness

**Recommendation**: Not ready for v1.0 publication. REL-002 remains NO-GO (independent gate).

The program delivered:
- SESSION-006 fix (P1 data-loss resolved)
- Plugin closure (existing implementation verified)
- Memory admission upgrade (evidence-backed Go)

Remaining gaps for a future pre-1.0 patch:
- External runtime remains primary (glm-5.2); zero qualifying Talos-primary sessions
- Plugin feature is opt-in (`wasm` feature); not part of default build
- No release, tag, publish, deploy, permission, or API change authorized

**Publication remains a separate maintainer decision.**

## 2026-07-17 Superseding Correction

The original packet is retained as historical evidence but is not the current acceptance truth.
Its I135 failure injection was synthetic, I136 did not expose a real loaded-package path, I137 did
not meet the published comparison/artifact requirements, and I138 applied an unsupported Go.

Current truth lives in I135-I140 and the execution package checkpoint: real persistence failure
injection, explicit local package loading with typed `/plugins` state, a byte-stable five-policy
No-Go report, restored production memory behavior, and SEC-001 exact external-path authorization.
This packet may be marked Complete again only after the corrective full locked ladder passes.

## 2026-07-17 Corrective Acceptance

The corrective ladder passed: locked fmt/check/Clippy/workspace tests, release preflight,
governance validation, and `git diff --check` are green on the corrected working tree. The
accepted implementation package is:

- I135: real persistence failure injection plus reconstructed durable transcript evidence.
- I136: explicit confined offline package loading, permission/provenance execution, and typed
  `/plugins` state.
- I137: a byte-stable five-policy benchmark selecting No-Go.
- I138: production memory baseline restored; published experimental API retained inert for semver
  compatibility; credential-shaped-content rejection remains independent.
- I140: exact external-path authorization accepted under ADR-047 and its security review.

The earlier Go/transition-notice/synthetic-failure claims remain historical and superseded.
I139 itself remains Review until these corrections are committed/pushed and replayed from clean
`main`. REL-002 remains NO-GO, and this acceptance does not publish a release.
