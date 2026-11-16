# SEARCH-001-D3-T1: Synthetic Bing HTML and Unadmitted-Host Negative Conformance

**Status**: Complete / Closed upon owner-first closeout publication — Implementation Completion Commit `9cbc615cebb119ead9706a4d00159d7151639606` (PR #944)
**Type**: Technical test-only Story
**Parent**: [SEARCH-001-D3 / #937](SEARCH-001-D3-bing-direct-html-feasibility.md); [SEARCH-001 Epic / #624](SEARCH-001-zero-config-global-search.md)
**Source Issue**: [#941](https://github.com/wjhuang88/talos/issues/941)
**Iteration**: [I326](../../iterations/I326-search-bing-synthetic-negative-conformance.md) — Complete / Closed upon owner-first closeout publication
**Baseline**: [D3 research / #938](../../reference/SEARCH-001-D3-BING-DIRECT-HTML-2026-11-13.md), merged `8e32fcadb5698e8a5cf35affd0a064ba8d02daf4`

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | Only native deterministic, Talos-authored synthetic Bing HTML and *negative* prepared-host/permission fixtures in `talos-tools`; add evidence/coverage matrix. **No production code, new endpoint or live provider request** |
| Claimed At | 2026-11-13 |
| Source Issue | #941 |
| Governance Claim PR | #943 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Governance claim+activation #943 merged `2f43e7d70701e91a41ddc0962b7768ec773ed5dc` after CI43426958043 and independent no-blocker review/CAS. Implementation #944 passed exact-head native CI43428872719 (8/8) and independent read-only security/API review, then merged with expected-head CAS at `9cbc615cebb119ead9706a4d00159d7151639606`. |
| Implementation PR | #944 merged `9cbc615cebb119ead9706a4d00159d7151639606` |
| Last Updated | 2026-11-13 |
| Handoff / Release Condition | Test-only slice accepted after implementation CI/review/CAS; close #941 only when owner-first closeout merges. New Bing live rights, query endpoints, real-region evidence and independent provider admission remain blocked/unproven. |

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
4. **Reachability is not search success**: characterize, only with existing
   synthetic/loopback probe and query fixture seams, the case where a Bing
   root HEAD/GET returns HTTP 200 but the query path fails or produces no
   admitted search result. Report whether this distinction is already tested;
   do not introduce new probes, health policy or production behavior.
5. **Preserved source family**: the compatibility parser's current compound
   DuckDuckGo backend ID is not renamed or reinterpreted as two native providers.
6. Exact-head locked targeted tests and applicable Linux/macOS/Windows
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
this completed I326 work touches only search **test-only** paths and its
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

## Completion Evidence — I326 / #941

Completion Commit: `9cbc615cebb119ead9706a4d00159d7151639606` — actual merged test implementation [PR #944](https://github.com/wjhuang88/talos/pull/944); this docs-only closeout is not the implementation SHA.

- **Claim and activation:** [PR #943](https://github.com/wjhuang88/talos/pull/943) merged `2f43e7d70701e91a41ddc0962b7768ec773ed5dc` after exact-head CI43426958043 and independent read-only governance review. The proposal became effective only on main.
- **Tests and receipt:** [PR #944](https://github.com/wjhuang88/talos/pull/944) adds exactly five Talos-authored synthetic `#[cfg(test)]` cases in three test modules and [coverage/known-gap evidence](../../reference/SEARCH-001-D3-T1-OFFLINE-COVERAGE-2026-11-13.md); no production code, Cargo, endpoint, permission or routing change. Its first CI43428188370 encountered one rustfmt failure and was cancelled; the accepted exact head `196e392b1acd0106762b1226787f2cb122ca1c4a` contains the one-line correction.
- **Exact-head native CI:** [CI43428872719](https://github.com/wjhuang88/talos/actions/runs/43428872719) **8/8 SUCCESS**, including macOS Format/Check/Clippy/Test, Windows workspace + Windows Rust, Linux/Windows Desktop, installer and remote Issue reconciliation. No individual native platform claim exceeds those jobs.
- **Independent security/API review:** [review 6853544090](https://github.com/wjhuang88/talos/pull/944#issuecomment-6853544090) and [re-review 6853572111](https://github.com/wjhuang88/talos/pull/944#issuecomment-6853572111) found no blocking scope, authority or semantic findings on the exact accepted head. Advisory reviews are not an invented human approval.
- **Merge-time CAS:** branch head and exact CI SHA matched; newer ARCH-035/#945 main changed only seven unrelated documentation paths, none of #944's three Rust test files or coverage document. GitHub confirmed squash merge `9cbc615cebb119ead9706a4d00159d7151639606`.
- **Retained limitation:** hand-authored `aside.b_ad` containing `li.b_algo` is selected by the current parser: a *documented false-organic-positive risk*, **not** fixed or production-accepted as organic. Alternate actual Bing DOM, `www.bing.com` and `bing.cn` live behavior/rights, mainland-China evidence and multi-native-provider admission remain BLOCKED/UNKNOWN. No additional search source or external query access was admitted.
- **Owner-first closure:** publish this source-linked closeout, then close source Issue #941. Parent research #937 and Epic #624 remain **Open** with D/#644 ZERO-ADMISSION and E/F/G/I gates intact. Rollback is reverting the test-only fixture changes, without runtime migration.
