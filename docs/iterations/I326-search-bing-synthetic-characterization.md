# Iteration I326: SEARCH-001-D3.2 Offline Bing Compatibility Characterization

> Document status: Active / Claimed — proposed atomic I326 claim via #942, **ineffective until merge; no implementation before acceptance**
> Published plan date: 2026-11-13
> Objective: fill proven gaps in Talos-owned synthetic **test-only** characterization of existing Bing HTML compatibility handling, without adding hosts, providers, runtime behavior or live automated searches.
> Baseline: D3.1 research [#938](https://github.com/wjhuang88/talos/pull/938) merged `8e32fcadb5698e8a5cf35affd0a064ba8d02daf4`; [CI43424338312](https://github.com/wjhuang88/talos/actions/runs/43424338312) success, independent review no blockers. This is documentary acceptance only.
> MVP deliverable: executable focused synthetic negative tests with exact test evidence and an explicit PASS/BLOCKED/UNKNOWN matrix, not additional search functionality.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed — proposed, ineffective until #942 merge |
| Responsible Actor | @wjhuang88 (proposed) |
| Executing Agent | ChatGPT / GPT-6 (proposed) |
| Work Slice | D3.2 only: inventory existing Bing parser/prepared compatibility tests against BF-01..BF-13 and add narrowly missing `#[cfg(test)]` synthetic HTML/local-transport characterization for ad/organic confusion, known-vs-unknown redirect authority, 200 empty/challenge differentiation, root-vs-query health caveats, cancellation/permission and preserved compatibility identity. No production edits or live Bing network. |
| Claimed At | 2026-11-13 (proposed) |
| Source Issue | #937 |
| Governance Claim PR | #942 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User authorized continued D3 after #938; exact scope and single-maintainer rationale recorded in governance #942. Required: exact-head CI, both governance validators, independent no-blocker review, main/claim/overlap and expected-head CAS. Claimed/Active proposal is ineffective until #942 merges. |
| Implementation PR | Not started |
| Last Updated | 2026-11-13 |
| Handoff / Release Condition | On accepted test-only stage, record test evidence and review, close this bounded iteration (not the D3 provider/right-to-use gate); retain all unverified live/terms/region questions under Issue #937. |

## Published baseline / dependency and iteration inventory

| Story | Parent | Status at selection | Dependency | Deliverable |
|---|---|---|---|---|
| [SEARCH-001-D3 / #937](../backlog/active/SEARCH-001-D3-bing-direct-html-feasibility.md) — D3.2 ONLY | [SEARCH-001 / #624](../backlog/active/SEARCH-001-zero-config-global-search.md) | D3.1 documentation accepted, D3 overall Ready/Unclaimed | Existing C/#643, L/#901 and D3.1/#938 accepted; BF offline design; **no rights gate needed for wholly Talos-authored local synthetic tests** | Tests/evidence for proven gaps, no production behavior |

Active/Review/Planned inventory retained independently: GIT-COMPAT-001 I324/#919, STORE-002 I325/#670, ARCH-035 I320/#897, PROVIDER-008 I323/#724 and TUI/Windows/other current owners keep their own claims/CI. I319 SEARCH-001-L is Complete/Closed. I326 has a unique test-only `talos-tools` search scope; none of the open Git, Storage, Server, Agent Session, TUI or Provider PRs owns this slice. Do not overwrite their owner/docs or broaden this into SearchRouter or protected network authority implementation. The maintainer's continued-search instruction selects this *separately owned, non-overlapping* limited slice without claiming exclusive global iteration ownership.

## Scope and precise acceptance

1. **Test inventory first**: identify existing source-test cases that already cover BF-01..BF-13 and document concrete uncovered cases. Existing `bing_parser/tests.rs` has selectors/cleanup, first-link, dedup, status, 429/503, challenges, empty, UTF-8/BOM, MAX_BODY_BYTES/MAX_FIELD_BYTES, count and redacted debug. Never duplicate already verified behavior.
2. **Organic-vs-decoy negative controls**: use strictly invented `li.b_algo`-like and advert/navigation markup to test whether an ad/foreign page can create a false organic result. An observed current limitation is an explicit result: failing assertions must not be hidden or marked passing; non-blocking redacted characterization may be published without changing production selectors.
3. **Authority/redirect/refusal characterization**: use synthetic URIs and in-memory/loopback-only fixtures with existing prepared request authority and per-hop checks. Prove unknown redirect target cannot silently become an allowed query destination, and that a head/root probe reaching HTTP 200 does not itself establish search-result success. Do not insert `www.bing.com` or `bing.cn` into any production manifest.
4. **Health/error/cancellation distinction**: existing frozen 200 valid/empty/challenge and 403/429 paths remain distinct; test only proven uncharacterized cases and capture whether missing health signals are a future design issue rather than manufacturing runtime health improvements. Network permission/cancellation denial must not send a query to unknown recipients.
5. **Evidence and constraints**: no genuine Bing/other upstream markup, direct public SERP network calls, new dependencies, provider/endpoint/URL allowlist additions, production HTTP/search/router/code changes, user-visible schema or key/paid fallback. Compare one logical Bing family, never count different Bing hosts as two native independent zero-key sources. No mainland-China reachability claim or vendor-rights conclusion.

## Exclusions, risks, rollback

- Live Bing automated access and use/output redistribution remain **BLOCKED** pending independent applicable-use verification under Microsoft agreement effective 2026-09-30. Offline invented fixtures are not proof that use is permitted or that templates match current live Bing.
- Do not touch `crates/talos-tools/src/search_backend/bing_parser.rs` production functions, default SearchRouter chain, `legacy_owned` runtime client, transport source/host configuration, `Cargo.toml`/lockfiles, Network permission/SSRF budget, browser/CDP, configuration, SDK or default search.
- Risk: synthetic fixtures could be misinterpreted as actual provider results, or a test could accidentally contact network. Require hardcoded `.invalid` URLs and in-memory test transport, redacted assertions, and explicit no-real-endpoint evidence.
- Rollback: revert only the new test cases and evidence from the future implementation commit; existing parser/transport/permission/runtime remain byte-identical.

## Planned validation and review

- No implementation before the finalized governance claim PR reaches `main`. Only one stable code candidate PR after local convergence, no CI-as-debugger workflow.
- Focused locked Cargo tests for changed `talos-tools` search tests (with exact filtered test counts), plus `cargo fmt --all -- --check`, relevant locked check/Clippy/test and complete applicable workspace CI.
- Both `scripts/validate_project_governance.sh .` and `scripts/validate_collaboration_claims.sh .`, remote owner reconciliation when governance changes.
- Independent read-only API/network/permission/parser review on exact head: check no new query host, parser/selection changes, real HTTP, false provider admission, runtime regressions or mislabeled tests.
- Compare exact source head, accepted I326 owner/claim, current main and unrelated open PR paths before guarded merge.

## Required reads

- [Owner](../backlog/active/SEARCH-001-D3-bing-direct-html-feasibility.md), [D3.1 source packet](../reference/SEARCH-001-D3-BING-DIRECT-HTML-2026-11-13.md), [#937](https://github.com/wjhuang88/talos/issues/937), [parent #624](https://github.com/wjhuang88/talos/issues/624).
- `AGENTS.md`, `docs/sop/REQUIREMENT-INTAKE.md`, `docs/sop/AGENT-COLLABORATION.md`, `docs/sop/START-ITERATION.md`, `docs/sop/TESTING.md`, `docs/sop/CHANGE-CONTROL.md`.
- `crates/talos-tools/src/search_backend/bing_parser.rs`, `bing_parser/tests.rs`, `search_backend/transport/compatibility/chain.rs`; existing prepared manifest and protection fixtures.
- ADR-072, ADR-085, ADR-090; D/#644 COMPLETE/BLOCKED, D2/#848 unclaimed, browser/#891 blocked, L/#901 Complete/Closed, NET-001/#199 boundary.

## Execution / evidence

| Date | Phase | Result |
|---|---|---|
| 2026-11-13 | Proposed I326 governance claim #942 | Owner and I326 both propose Claimed/Active on this PR; no effective authority until #942 merges after exact-head CI, both governance validators, independent review and CAS. No test implementation or provider admission. |

**Not Complete**: No implementation/Completion Commit yet. D3.1 #938 documentation merge is historical evidence only, not this test iteration's implementation completion. Any later technical test stage must independently pass and leave provider rights/live/regional admission outside I326.
