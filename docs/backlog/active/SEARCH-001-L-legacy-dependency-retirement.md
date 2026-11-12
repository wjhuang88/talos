# SEARCH-001-L: Retire Legacy Rust-Websearch Build Dependency

**Status**: Complete / Closed upon owner-first governance closeout publication — Implementation Completion Commit `3cfe8927ac7c71976e8cb3aa02a44ad21c3ca970` (PR #907).
**Source Issue**: [#901](https://github.com/wjhuang88/talos/issues/901).
**Parent**: [SEARCH-001 / #624](SEARCH-001-zero-config-global-search.md).
**Selected iteration**: [I319](../../iterations/I319-search-legacy-dependency-retirement.md) — Complete / Closed upon closeout merge.
**ADR gate**: [ADR-085 A1](../../decisions/085-talos-owned-search-routing-boundary.md#amendment-a1--equivalent-build-package-retirement-2026-11-10-accepted-via-902), accepted via merged #902.
**Source truth**: [SEARCH-001-I retirement preflight](../../reference/SEARCH-001-I-DEPENDENCY-RETIREMENT-PREFLIGHT-2026-11-10.md), published in #900.
**Completed predecessor**: [SEARCH-001-K / #850](SEARCH-001-K-compat-fallback-ownership.md) (I317/#860).

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | SEARCH-001-L L1: replace the residual `rust-websearch` dependency in the dormant/public legacy `WebSearchTool` and test oracle, preserve API/behavior, then remove Cargo and current fixture locked package references only with independently verified tests |
| Claimed At | 2026-11-10 |
| Source Issue | #901 |
| Governance Claim PR | #902 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance claim/ADR-085 A1 #902 merged; frozen upstream oracle #904 merged; implementation #907 passed exact-head full native CI4121 8/8, independent security/API source review and expected-head CAS squash merge. |
| Implementation PR | L1a #904 merged; L1b [#907](https://github.com/wjhuang88/talos/pull/907) merged `3cfe8927ac7c71976e8cb3aa02a44ad21c3ca970` |
| Last Updated | 2026-11-12 |
| Handoff / Release Condition | L1 implementation, CI and review complete; publish this owner-first closeout and then close #901. No new provider admission, global search gate, parent #624 closure or release acceptance. |

## Exact dependency and non-overlap

This is **dependency-package retirement only**, **not** SEARCH-001-I's G-dependent final
global-search provider retirement acceptance. The parent I's G/H gate, D/#644
ZERO-ADMISSION, E/F/G blocked, D2/#848 and native browser #452 remain untouched.
#901 owns the standalone legacy `WebSearchTool` + upstream test-oracle removal and Cargo
closure, not the protected SearchRouter production parser/helpers/transport already completed
by K/#850. The outdated alternative K Draft #861 is not an additional current
implementation claim; it must not be merged or overwrite #860.

The accepted protected Auto route remains Lite → DDG Instant Answer → DDG HTML →
Bing CN → Wikipedia *knowledge* fallback under existing Network permission and
endpoint authority. No new provider, runtime fallback to unprepared direct execution,
credential discovery, stealth/challenge bypass, query-bearing startup probes or
generic Net resilience change is part of L1.

## Runnable work slice L1

1. Freeze offline deterministic pinned-upstream HTML/entity/redirect/UA and legacy
   fixed-host transport provenance **before** removing the actual upstream-oracle calls.
2. Preserve exported `WebSearchTool::new()`, constructor, `AgentTool` JSON schema,
   result formatting, zero-key ambient isolation, errors and behavior as an
   explicitly reviewed owned implementation; retain stand-alone authority constraints.
3. Eliminate `rust_websearch` source calls in all supported build/test paths.
4. Remove `dep:rust-websearch` from `talos-tools/network` and the direct Cargo
   dependency, regenerate root and actively exercised external fixture locks using Cargo,
   and verify the package does not enter normal/coding/all-features dependency closure.
5. Retain the MIT license and derivative copyright headers and report any observed
   legacy/API/protocol variance or unresolved unsupported fixture.

## Acceptance and review

- `cargo fmt --all -- --check`, `cargo check -p talos-tools --features network --locked`,
  `cargo test -p talos-tools --features network --locked`,
  `cargo clippy -p talos-tools --features network --all-targets --locked -- -D warnings`.
- Frozen regression vectors cover HTML entities, malformed UTF-8/redirect,
  12 User-Agent byte pool, old public `WebSearchTool`, test-only proxy,
  failure/empty, result order/format, permission/endpoint, timeout/cancel and Wikipedia.
- Full supported workspace/CLI/SDK fixture build and current external locked Cargo graphs;
  native Windows/Linux/macOS CI and independent network/API/security review at exact head.
- If full removal fails public compatibility, stop and record blocked difference.
  Merely moving the dependency to an opt-in legacy feature does not satisfy full removal.
- Preserve parent #624 Open, G-dependent SEARCH-001-I Blocked and #850 Closed.

## Rollback

Use version/deployment rollback to known-good `main` prior to L1.
Do not restore an invisible runtime fallback from protected SearchRouter into
unprepared legacy `WebSearchTool` and do not change provider selection to work
around a build graph. Do not claim success until a merged implementation SHA exists.

## Implementation and acceptance checkpoint (2026-11-11)

- Governance claim and ADR-085 A1: [#902](https://github.com/wjhuang88/talos/pull/902), merged at `135fd80508896c9bd3fa30879d4c125fc272a444`.
- L1a independent pinned-upstream oracle: [#904](https://github.com/wjhuang88/talos/pull/904), merged at `00e4ebdd0da0b683005e115e199d6ffd298c2aa1`.
- L1b implementation: [Draft #907](https://github.com/wjhuang88/talos/pull/907), **not yet merged or accepted**. Treat the PR HEAD and its exact-head CI/review as authoritative, not any historical candidate SHA in a dated checkpoint. This slice removes `rust-websearch` and `reqwest 0.12` from the three Cargo-generated locks, retains MIT/public legacy behavior, uses isolated reqwest 0.13/WebPKI/compressed decoding, and keeps protected SearchRouter untouched.
- Transport compatibility review found two ambient system proxy paths after Cargo feature unification: legacy DDG and Wikipedia fallback. The first is fixed by client-level `.no_proxy()`; the second is fixed at the Wikipedia client as of the same candidate. Both have isolated subprocess proxy-disclosure regressions. Compression/TLS behavior remains isolated to the legacy client.
- Historical full native [CI4063](https://github.com/wjhuang88/talos/actions/runs/42973360173) passed 8/8 **before** the concurrent limiter fix. [Temporary locked-graph regeneration and focused four-waiter test](https://github.com/wjhuang88/talos/actions/runs/43007470234) succeeded after syncing the newer Gemini main. These are supporting evidence only; **final code acceptance requires CI on the current #907 HEAD**, including remote owner reconciliation after merged #915.
- Independent review found and prompted fixes for compressed response handling, header-first non-2xx errors, HTML decoding, WebPKI roots, DDG and Wikipedia ambient proxy routing, and global concurrent request spacing. Current limiter holds a fair Tokio mutex through its jittered wait, with a four-waiter regression; post-fix source review reported no further concrete code blocker. The dated checkpoints are historical; final verification remains the exact-head CI/review linked from #907.
- Continue to hold #901 Open and #907 Draft until current-head CI, independent technical/governance review, dependency closure and a successful CAS merge; parent #624 and SEARCH-001-I E/F/G/I gating stay unchanged.

## Final completion and acceptance — 2026-11-12

Completion Commit: `3cfe8927ac7c71976e8cb3aa02a44ad21c3ca970` — actual implementation #907; this documentation closeout is not the implementation commit.

- **Implementation merge evidence:** `3cfe8927ac7c71976e8cb3aa02a44ad21c3ca970` from merged [PR #907](https://github.com/wjhuang88/talos/pull/907). Exact accepted source head: `faa4982caed19e3f35b9e5648c47c36184c24f52` on `main@1eabf2964b5a81005f626644bd99053000518fdb`, squash-merged using expected-head CAS.
- **Native acceptance:** [CI4121](https://github.com/wjhuang88/talos/actions/runs/43195189353), all **8/8 SUCCESS**, including Windows workspace + Windows Rust workspace, macOS format/check/Clippy/test, Linux and Windows Desktop, installer, classifier and remote Issue reconciliation. Isolated [bounded response regression](https://github.com/wjhuang88/talos/actions/runs/43194684946) and [Cargo/limiter test](https://github.com/wjhuang88/talos/actions/runs/43007470234) also succeeded; temporary testing workflows never entered #907.
- **Independent security/API review:** [exact-head review on accepted source](https://github.com/wjhuang88/talos/pull/907#issuecomment-6820413223) found no concrete source/routing/dependency/owner/ADR blocker after iterative findings were corrected. Inline review threads resolved/outdated. Review is read-only, not an invented human signoff.
- **Delivered:** public zero-key standalone `WebSearchTool` compatible with frozen #904 upstream oracle; no `rust-websearch` nor `reqwest 0.12` in the three active Cargo locks; isolated reqwest 0.13/WebPKI + gzip/br/deflate handling; DDG/Wikipedia ambient proxy privacy isolation; 4-waiter process-global rate-limit regression. Non-2xx responses are rejected before the body; legacy response wire consumption uses chunked reading with max **8 MiB accepted raw bytes**, plus max **8 MiB decoded output**.
- **Explicit bound limitation:** Reqwest may allocate a transient transport chunk before our check, and `Vec` capacities can exceed logical lengths. The 8 MiB checks are **not a proven hard process peak-memory/RSS ceiling**. No new source/network permission or protected SearchRouter behavior, region coverage, E/F/G/I admission or release qualification was granted.
- **Rollback:** roll back to a known pre-L1 application/version commit; do not silently reenable third-party legacy execution inside the protected prepared route. Completion is bounded to #901; [parent #624](https://github.com/wjhuang88/talos/issues/624) stays Open, SEARCH-001-I remains G-dependent/Blocked and predecessor #850 stays Closed.
- **Governance sequence:** code #907 merged; this document-only owner-first closeout must merge and synchronize #901 **before** final issue closure. This documentation merge is not a replacement for the implementation Completion Commit.
