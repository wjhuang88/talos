# Iteration I326: Bing Synthetic Negative Conformance (SEARCH-001-D3-T1)

> Document status: Complete / Closed upon owner-first closeout publication — implementation #944 merged
> Published plan date: 2026-11-13
> Baseline: SEARCH-001-D3/#937 documentary source packet accepted via #938 (`8e32fcadb5698e8a5cf35affd0a064ba8d02daf4`).
> Planned objective: native offline regression characterization for existing Bing HTML parser and unadmitted host/redirect authority, **without production or network behavior change**.
> MVP: deterministic test-only negative fixtures with a source-linked known-gaps receipt.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | SEARCH-001-D3-T1/#941: bounded synthetic parser and host/permission *test-only* assertions and evidence receipt; no live Bing, new endpoint or production changes |
| Claimed At | 2026-11-13 |
| Source Issue | #941 |
| Governance Claim PR | #943 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | #943 claim/activation merged `2f43e7d70701e91a41ddc0962b7768ec773ed5dc` after exact-head CI43426958043/review/CAS; test implementation #944 accepted on exact head `196e392b1acd0106762b1226787f2cb122ca1c4a` after CI43428872719 8/8 and independent review/CAS, merged `9cbc615cebb119ead9706a4d00159d7151639606`. |
| Implementation PR | #944 merged `9cbc615cebb119ead9706a4d00159d7151639606` |
| Last Updated | 2026-11-13 |
| Handoff / Release Condition | Bounded T1 test implementation accepted; release I326 claim upon owner-first closeout merge and then close #941. No permission to perform live Bing queries, new routing or E/F/G/I admission. |

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

Claim/activation [PR #943](https://github.com/wjhuang88/talos/pull/943) merged at `2f43e7d70701e91a41ddc0962b7768ec773ed5dc` after exact-head CI43426958043 SUCCESS, independent read-only review and expected-head CAS. The I326 claim became effective on main before any #944 test code was committed.

## Completion evidence

Completion Commit: `9cbc615cebb119ead9706a4d00159d7151639606` — merged [implementation PR #944](https://github.com/wjhuang88/talos/pull/944) (test-only). The later documentation closeout is **not** the implementation completion commit.

- Exact accepted head: `196e392b1acd0106762b1226787f2cb122ca1c4a`.
- [CI #43428872719](https://github.com/wjhuang88/talos/actions/runs/43428872719) **SUCCESS, 8/8 jobs**; macOS full-format/check/clippy/test, Windows native and Rust workspace, Linux/Windows Desktop, classifier, installer and Issue owner reconciliation. First run #43428188370 failed rustfmt and was cancelled; its one-line fix was accepted on final head.
- Independent read-only [security/API review](https://github.com/wjhuang88/talos/pull/944#issuecomment-6853544090) and [exact-head re-review](https://github.com/wjhuang88/talos/pull/944#issuecomment-6853572111): no blockers.
- Added five pure synthetic negative tests in existing `#[cfg(test)]` modules; [coverage receipt](../reference/SEARCH-001-D3-T1-OFFLINE-COVERAGE-2026-11-13.md) names both preexisting and new tests. No production parser, transport, source order, Network/ADR authority or Cargo edits. Existing `cn.bing.com` compatibility preserved; `www.bing.com` and bare `bing.cn` remain unadmitted.
- **Residuals:** sponsored wrapper with `li.b_algo` remains a known classifier false positive; no actual Bing DOM/redirect, China coverage, Microsoft rights approval or native provider identity was established. D3/#937 remains research-only; #624 stays Open, E/F/G/I gates unchanged.

## Variance And Residuals

An initial formatting-only CI failure required one exact rustfmt fix; no test semantics changed. The all-success exact-head CI and owner-first closure supersede older draft statuses. Actual source evidence is the prior implementation commit, not this documentation status edit.

## Retrospective

The narrow BF gap audit prevented duplicate parser tests and captured a real structural false-positive risk without expanding search authority. Synthetic HTTP 403 under query-free reachability proves the failure is not represented as successful results, **not** that Bing is regionally reachable or distributable.
