# Iteration I326: Bing Synthetic Negative Conformance (SEARCH-001-D3-T1)

> Document status: Planned — proposed Active, ineffective until atomic governance claim merges
> Published plan date: 2026-11-13
> Baseline: SEARCH-001-D3/#937 documentary source packet accepted via #938 (`8e32fcadb5698e8a5cf35affd0a064ba8d02daf4`).
> Planned objective: native offline regression characterization for existing Bing HTML parser and unadmitted host/redirect authority, **without production or network behavior change**.
> MVP: deterministic test-only negative fixtures with a source-linked known-gaps receipt.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed — proposed Claimed; ineffective until governance merge |
| Responsible Actor | @wjhuang88 (proposed) |
| Executing Agent | ChatGPT / GPT-6 (proposed) |
| Work Slice | SEARCH-001-D3-T1/#941: bounded synthetic parser and host/permission *test-only* assertions and evidence receipt; no live Bing, new endpoint or production changes |
| Claimed At | 2026-11-13 (proposed) |
| Source Issue | #941 |
| Governance Claim PR | Pending — backfill Draft number |
| Authorization Mode | Single-maintainer merge (proposed) |
| Authorization Evidence | User continuation of D3 after accepted #938; pending exact-head CI, independent review and merge-time CAS |
| Implementation PR | Not started |
| Last Updated | 2026-11-13 |
| Handoff / Release Condition | Effective only after governance merge; revert/leave unclaimed if blocked by overlapping claim or validation. |

## Selected Story

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| [SEARCH-001-D3-T1 / #941](../backlog/active/SEARCH-001-D3-T1-bing-synthetic-negative-conformance.md) | D3/#937, Epic/#624 | Ready / proposed activation | D3.1 documentary PR #938 Complete | Native synthetic negative-conformance tests only; preserve existing network authority and behavior |

## Scope / non-goals

- Work solely under `#[cfg(test)]` parser / existing compatibility
  transport test seams, and a specific gap/fixture-evidence document.
- Cover ad/organic negative structures, DOM drift under locale variation,
  query-path failure despite root reachability, and refusal to treat
  `www.bing.com`/`bing.cn` aliases as authorized query hosts.
- Where existing tests already cover a BF case, record exact file/test names;
  avoid cloning old test code and do not assert a new Bing DOM capability.
- Preserve request authority, error/result semantics, public API,
  `cn.bing.com` compatibility, dependency graph, routing, provider ownership,
  URL admission and Network permission. No actual Bing traffic.
- Do not create any broad new transport tests that alter approved code;
  if an assertion requires changes outside `#[cfg(test)]`, record a blocker
  and propose an independently reviewed future slice.

## Native planned validation

- `cargo fmt --all --check`.
- `cargo test -p talos-tools --features network --locked bing_parser`
  (adjust exact filter to existing test paths; record execution counts).
- Existing focused `search_backend::transport` test/fixture regressions
  with `--locked`, isolated fixed/invalid hosts only, and safe query redaction.
- `cargo check --workspace --locked`, `cargo clippy --workspace --locked -- -D warnings`,
  `cargo test --workspace --locked` and supported cross-platform CI, subject
  to repository's actual stable changed-path classifier.
- Both governance validators, remote #941 status reconciliation, independent
  API/network/permission/test review, merge-time expected-head CAS.

## Dependency inventory / parallel selection decision

I324/#919 gix-preflight, I325/#670 local storage, I320/#897 host
composition, other active/review Desktop, Provider and TUI items
remain independently owned. This **test-only** I326 is disjoint in
Rust paths and service permissions. Its activation via a separate
atomic claim PR makes no other iteration Complete, paused or superseded,
and transfers no claim or review authority. No Epic/#624 implementation
activation. Authorizing parallel work means only this bounded distinct
slice, conditional on no latest-main/PR overlap at merge.

## Risks and rollback

- Incorrectly labelling sponsored results as organic: characterize
  synthetic parser behavior and document any gap; do not silently alter
  production parser classification.
- False network reachability/rights inference: no live request; preserve
  blocked default-use admission and region uncertainty.
- CI failure/operating-system variance: retain diagnostics, do not
  mark Complete or suppress tests.
- Revert introduced `#[cfg(test)]` fixtures/receipts only; no runtime
  migration or user configuration rollback.

## Activation / actual execution

Proposed as a governance-only PR; **no effective claim or code authority**
until a finalized PR has been reviewed, exact-head CI passed and a
single-maintainer expected-head CAS merge has succeeded.

## Completion evidence

No implementation Completion Commit yet. Governance claim alone never
completes T1 or SEARCH-001-D3/#937, nor admits default Bing routes.
