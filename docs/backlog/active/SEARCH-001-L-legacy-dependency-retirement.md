# SEARCH-001-L: Retire Legacy Rust-Websearch Build Dependency

**Status**: Active / Claimed (effective via merged governance #902; code PR #907 under review).
**Source Issue**: [#901](https://github.com/wjhuang88/talos/issues/901).
**Parent**: [SEARCH-001 / #624](SEARCH-001-zero-config-global-search.md).
**Selected iteration**: [I319](../../iterations/I319-search-legacy-dependency-retirement.md) — Active / Claimed.
**ADR gate**: [ADR-085 A1](../../decisions/085-talos-owned-search-routing-boundary.md#amendment-a1--equivalent-build-package-retirement-2026-11-10-accepted-via-902), accepted via merged #902.
**Source truth**: [SEARCH-001-I retirement preflight](../../reference/SEARCH-001-I-DEPENDENCY-RETIREMENT-PREFLIGHT-2026-11-10.md), published in #900.
**Completed predecessor**: [SEARCH-001-K / #850](SEARCH-001-K-compat-fallback-ownership.md) (I317/#860).

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | SEARCH-001-L L1: replace the residual `rust-websearch` dependency in the dormant/public legacy `WebSearchTool` and test oracle, preserve API/behavior, then remove Cargo and current fixture locked package references only with independently verified tests |
| Claimed At | 2026-11-10 |
| Source Issue | #901 |
| Governance Claim PR | #902 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance claim/ADR amendment are effective after #902 merged; #904 frozen upstream oracle is merged. This does not imply approval of Draft implementation #907; independent API/network/permission/security review and exact-head cross-platform CI remain mandatory. |
| Implementation PR | L1a #904 merged; L1b [#907](https://github.com/wjhuang88/talos/pull/907) Draft / unmerged |
| Last Updated | 2026-11-11 |
| Handoff / Release Condition | Governance is effective; code PR #907 requires exact-head CI and independent review, merge-time CAS and post-merge evidence before #901 can close. No provider admission or parent closure. |

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
- L1b implementation: [Draft #907](https://github.com/wjhuang88/talos/pull/907), code candidate `919440cb6269a6cff87a709bc954e79d878143df` at this checkpoint; **not yet merged or independently accepted**. Removes `rust-websearch`, retains MIT notice and public legacy surface, migrates the old standalone Tool and probe from `reqwest 0.12` to `reqwest 0.13`, regenerates root and both external fixture locks, keeps protected SearchRouter untouched.
- Transport compatibility review found two ambient system proxy paths after Cargo feature unification: legacy DDG and Wikipedia fallback. The first is fixed by client-level `.no_proxy()`; the second is fixed at the Wikipedia client as of the same candidate. Both have isolated subprocess proxy-disclosure regressions. Compression/TLS behavior remains isolated to the legacy client.
- Prior exact-head [CI4041](https://github.com/wjhuang88/talos/actions/runs/42849619095) succeeded 8/8 **before** the Wikipedia fix. [CI4048](https://github.com/wjhuang88/talos/actions/runs/42948624184) is pending for that newer code. No final acceptance is inferred from older CI.
- Continue to hold #901 Open and #907 Draft until new exact-head CI, independent read-only API/network/security review, explicit dependency/fixture closure and successful merge; parent #624 and SEARCH-001-I E/F/G/I gating stay unchanged.
