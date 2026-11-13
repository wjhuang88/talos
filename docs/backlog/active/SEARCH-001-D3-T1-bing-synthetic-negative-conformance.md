# SEARCH-001-D3-T1: Synthetic Bing HTML and Unadmitted-Host Negative Conformance

**Status**: Ready / proposed In Progress; **ineffective until governance PR merge**
**Type**: Technical test-only Story
**Parent**: [SEARCH-001-D3 / #937](SEARCH-001-D3-bing-direct-html-feasibility.md); [SEARCH-001 Epic / #624](SEARCH-001-zero-config-global-search.md)
**Source Issue**: [#941](https://github.com/wjhuang88/talos/issues/941)
**Iteration**: [I326](../../iterations/I326-search-bing-synthetic-negative-conformance.md), proposed only
**Baseline**: [D3 research / #938](../../reference/SEARCH-001-D3-BING-DIRECT-HTML-2026-11-13.md), merged `8e32fcadb5698e8a5cf35affd0a064ba8d02daf4`

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed — proposed Claimed; ineffective until reviewed governance PR merge |
| Responsible Actor | @wjhuang88 (proposed) |
| Executing Agent | ChatGPT / GPT-6 (proposed) |
| Work Slice | Only native deterministic, Talos-authored synthetic Bing HTML and *negative* prepared-host/permission fixtures in `talos-tools`; add evidence/coverage matrix. **No production code, new endpoint or live provider request** |
| Claimed At | 2026-11-13 (proposed; no effective claim yet) |
| Source Issue | #941 |
| Governance Claim PR | Pending — backfill after Draft PR creation |
| Authorization Mode | Single-maintainer merge (proposed; not effective until accepted) |
| Authorization Evidence | Maintainer instruction to continue D3 after #938 acceptance; exact-head governance CI and independent review required |
| Implementation PR | Not started |
| Last Updated | 2026-11-13 |
| Handoff / Release Condition | Activate only after governing PR merges; return to Unclaimed if not accepted. Future rights or new endpoint admission remains separate and blocked. |

## Goal / Value

Build a **small native offline regression evidence slice** for the observed gap between
a browser's direct Bing HTML and Talos' already-shipped Bing CN compatible parser.
Characterize *false organic positives/negatives and unapproved host admission*
rather than assuming `www.bing.com`, `cn.bing.com`, or `bing.cn` differ in a
measured way. Do not copy existing BF fixtures or weaken previously accepted behavior.

## Affected code and work slice

- Test-only lines in `crates/talos-tools/src/search_backend/bing_parser/tests.rs`;
  optionally **existing test-only transport/compatibility modules** if exact
  host and redirect permission negatives can be expressed with current test seams.
- A small dated offline coverage/limitations receipt under `docs/reference/`.
- No production `bing_parser.rs`, `transport`, `chain`, `SearchRouter`,
  `legacy_owned`, Cargo.toml/lock or model-facing schema change.
- Preserve the existing compatibility order
  DDG Lite → Instant Answer → DDG HTML → Bing CN and fixed query recipient
  `https://cn.bing.com/search`.

## Minimum focused acceptance

1. **BF coverage inventory** names existing tests already proving statuses,
   challenge text, strict UTF-8/BOM, field/DOM bound, ordered rows, fixed request
   parameters, chained cancellation and existing `cn.bing.com` behavior.
   Do not duplicate covered assertions simply to raise a test count.
2. **Synthetic adversarial HTML**: advertising-only container vs organic
   `li.b_algo`, mixed ad/organic with false-positive risk, deliberately
   changed selectors/locale, 200 empty or challenge; distinguish
   *characterized parser behavior* from a promise of future Bing template parity.
   A current known misclassification may be documented, not silently fixed
   or mislabeled successful organic admission.
3. **Fixed-host authority**: synthetic `www.bing.com`, bare `bing.cn` and
   unknown cross-origin redirects must not be silently promoted to authorized
   query-bearing hosts in the current prepared manifest. Validate via
   existing host/redirect test seams only, never any real Bing request.
4. **Preserved source family**: the compatibility parser's current compound
   DuckDuckGo backend ID is not renamed or reinterpreted as two native providers.
5. Exact-head locked targeted tests and applicable Linux/macOS/Windows
   CI, independent read-only security/API/test review and CAS merge. No skipped
   failing tests; keep observable blockers and coverage limitations explicit.

## Explicit exclusions

No query-bearing or probe call to Bing, third-party HTML capture, a native
Browser/CDP connector, CAPTCHA bypass, runtime host allowlist change, routing
priority, health policy implementation, new provider backend, API key, network
permission, proxy, redirect or SSRF exception, startup request or country/GeoIP
routing. Do not treat code tests or public endpoint reachability as evidence
of Microsoft authorization, independent indexes, mainland-China network
availability or SEARCH-001 E/F/G/I admission.

The 2026-09-30 Microsoft Bing/MSN product-use rights remain **BLOCKED /
UNPROVEN** for new automated distributed-default requests. Testing synthetic
input cannot clear this gate. Existing SEARCH-001-D / #644 ZERO-ADMISSION,
D2/#848, Browser/#891, completed C/H/K/L and Epic #624 remain unchanged.

## Dependency / iteration overlap assessment

D3.1 source packet #938 was accepted and merged. I319 SEARCH-001-L is
Complete/Closed; I314 H and I293 C are Complete/Closed. Unrelated
I324/#919 Git preflight, I325/#670 storage D, I320/#897 composition and
other Review/Active Desktop/TUI owners remain independently governed;
this proposed I326 work touches only search **test-only** paths and its
own documents. Explicit parallel selection is bounded to this nonoverlapping
test slice, not a broad priority or implementation transfer.

## State / status owners and required reads

- This owner defines test scope and completion; I326 defines execution; source
  Issue #941 handles discussion; D3/#937 remains research intake/rights gate;
  Epic #624 owns terminal search acceptance.
- `AGENTS.md`; `docs/sop/AGENT-COLLABORATION.md`,
  `START-ITERATION.md`, `REQUIREMENT-INTAKE.md`, `CHANGE-CONTROL.md`.
- `crates/talos-tools/src/search_backend/bing_parser.rs` and
  `bing_parser/tests.rs`; `transport/compatibility/chain/legacy_parity_tests.rs`,
  `transport/tests/headers.rs`, and ADR-072/085/090.
- No implementation until finalized collaboration claim + iteration activation
  is **merged** to target branch; all Draft PR content is a proposal only.
- Rollback: revert independently added test-only assertions and receipts.
  The runtime never changes. If independent review identifies behavior
  needing parser/transport fixes, document a separate future scope.

## Completion evidence (pending)

Implementation Completion Commit: **not yet available**.
A governance-only merge is not completion. Retain In Progress/Review until
real test execution/independent review/owner-first closeout evidence exists.
