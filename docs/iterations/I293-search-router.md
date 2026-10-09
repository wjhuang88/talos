# Iteration I293: Talos-Owned SearchRouter

> Document status: Active
> Parent: SEARCH-001-C / #643 / SEARCH-001 / #624
> Objective: implement the accepted ADR-085 first-valid-success router after B, without changing the model-facing contract.

| Field | Value |
|---|---|
| Story | SEARCH-001-C |
| Source Issue | #643 |
| Depends on | Accepted ADR-085; SEARCH-001-B / #642 complete in #656; NET-001 boundary coordination |
| Claim State | Claimed |
| Implementation PR | Not started |
| Completion | Pending |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | GPT-6 Sol / talos开发 session |
| Work Slice | SEARCH-001-C Talos-owned SearchRouter behavior and deterministic characterization only |
| Claimed At | 2026-10-06 |
| Source Issue | #643 |
| Governance Claim PR | #664 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance-only claim+activation PR; exact-head CI, governance validators, remote Issue reconciliation and merge-time CAS required. |
| Last Updated | 2026-10-09 |
| Handoff / Release Condition | Claim effective after #664 merge `d801790454e252699684253166984662ea9dab6a`; implementation remains bounded by ADR-085 and this iteration. |

## Scope

- Add Talos-owned first-valid-success routing with bounded hedging and cancellation/deadline propagation.
- Preserve backend identity and typed failures, while keeping health state bounded and query-free.
- Keep paid providers explicit opt-in, Wikipedia as compatibility knowledge fallback, and rust-websearch as rollback adapter.

## Non-goals

No model-facing schema/output change, GeoIP or country routing, startup probe, generic retry/circuit-breaker system, new public crate, or provider admission.

## Acceptance

Deterministic fixtures cover first-valid-success, fast-error/slow-success, all-fail, invalid result, timeout, cancellation and bounded hedging. Locked tests, governance validation, focused review and rollback evidence pass.

## Test-Only Characterization Stage — 2026-10-09

Base: `270911c071961a79c85af742a4daf62d4ad6735a`. This stage preserves production search
behavior and the model-facing contract. Both new tests are inside the existing test module:

- Empty, empty-URL and non-HTTP URL responses returned as `Ok` can preempt a later valid response
  in the current select pattern. This records a gap against ADR-085, not first-valid-success.
- A successful or error winner drops the owned pending loser future. This is local cleanup
  evidence only; it does not prove HTTP cancellation or abortion of dependency-spawned work.

An existing backend test uses an array instead of a fixed vector to satisfy pinned Clippy's
`useless_vec` lint. No runtime wiring, provider admission, manifest or lockfile changes are made.
The older disposable two-future specification is not used as production evidence.

The previous unpublished local checkpoint `d0c8906dcca9f98b01d563818f395e2e4e3774d5` passed
search tests 25/25, tool-library tests 113/113, targeted Clippy including network tests, workspace
check/Clippy and both governance validators. Its first full preflight exhausted the 32 GiB
filesystem; compact local builds then reached workspace tests but failed in the unchanged runtime
Unix socket fixture with `PermissionDenied / Operation not permitted` (64 passed / 1 failed).
These are historical results, not a full-preflight pass for the reconstructed candidate. The
unpublished checkout/cache was subsequently lost during workspace maintenance. This stage is
reconstructed and freshly verified; the previous commit is not claimed as pushed evidence.

The maintainer explicitly authorized skipping that single environment-blocked fixture locally
on 2026-10-09 and retaining its complete PR CI coverage. The local workspace test command is:

```bash
cargo test --locked --workspace -- --skip tests::runtime_evidence_rejects_directory_symlinks_and_socket_entries
```

Local build settings are `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0`
to bound storage. These do not change repository profiles. No `#[ignore]`, test deletion, permission
policy change or CI skip is introduced. Merge still requires exact-head full CI without this skip
and applicable review. Local validation results are recorded below before submission.

Fresh reconstructed-tree validation: `cargo test --locked --workspace` with the single explicit
skip above exited 0: 111 suite summaries, 3455 passed, zero failed/ignored, one filtered test.
This is not an unskipped full-workspace pass. Format, whitespace and both governance validators
also passed.
Workspace check and workspace Clippy (`--locked`, `-D warnings`), targeted network-feature test
Clippy, and the external Runtime SDK fixture in both default and coding modes subsequently passed.
Public-site, installer, text-boundary and CI-classifier checks passed as well. The local socket
skip remains the only deferred test; exact-head unskipped CI and merge review remain pending.

This stage cannot complete C: actual routing, typed failures, invalid-result filtering, bounded
hedging, enclosing deadline/cancellation, bounded query-free health, NET-001 coordination and
binary integration/rollback evidence remain pending. I293 remains Active / Claimed, completion
pending; #643 and #624 stay open. D business correspondence is not required for offline tests.
