# I335: Bing Dual Host Real Transport + Explicit Opt-in (SEARCH-001-D3-T5)

> Document status: Complete / Closed for bounded I335 source implementation **upon owner-first closeout merge**; external #1000 rights and #957 live/default remain unapproved.
> Selection: 2026-11-22. Maintainer prioritizes working functionality over supplier-rights research.
> Goal: Complete an executable, secure, permission-aware WWW-first/CN-fallback search path without changing the default network route or equating implementation with licensing.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | I335 / D3-T5 #1001: real transport, explicit opt-in, deterministic offline transport fixtures; no default/live admission |
| Claimed At | 2026-11-22 |
| Source Issue | #1001 |
| Governance Claim PR | #1002 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer directed code-only implementation 2026-11-22; claim #1002 merged `1b3fc18bc323ffe71603766ecc6c0e61afa13412`; source #1003 exact-head [CI 8/8](https://github.com/wjhuang88/talos/actions/runs/45160764389), independent [APPROVE](https://github.com/wjhuang88/talos/pull/1003#issuecomment-7103434725) and merge CAS; #1000 rights separate. |
| Implementation PR | [#1003 MERGED](https://github.com/wjhuang88/talos/pull/1003); **Completion Commit: `bab2bdf31fc8200758397d38077dab57b9dd1aa4`** |
| Last Updated | 2026-11-23 |
| Handoff / Release Condition | I335 source Complete / Closed upon governance closeout publication and #1001 Issue synchronization; Gate-0/#1000, #957 live/default, #937/#624 remain separately Open/Blocked |

## Selected Story / current owner inventory

[SEARCH-001-D3-T5](../backlog/active/SEARCH-001-D3-T5-bing-dual-runtime.md), [Issue #1001](https://github.com/wjhuang88/talos/issues/1001). I329 T3a and I333 T4 are Complete and unchanged; T3b/#957 production live/default still BLOCKED/Unclaimed. #1000 access/output rights and #937/#624 remain open. No duplicate technical feasibility spike.

## Invariants, tests and handoff

1. Reuse the fixed two-host I333 preprepared query plan, real `SearchTransport`, owned parser/admission, one ToolNetworkManifest/authority, proxy, budget, deadline and explicit opt-in.
2. Existing default Auto's DDG-first/CN-only order and permission hosts stay byte-for-byte stable; no automatic or ambient WWW enablement. Any new opt-in must independently scope both host query recipients.
3. Test real transport via local mock DNS/HTTPS servers with allowed typed fallbacks and forbidden HTTP/challenge/redirect/permission/deadline/budget outcomes. No Bing internet CI.
4. Record exact native CI/independent security review/CAS and rollback. After source merge, owner-first status closeout cites existing source Completion Commit; #1000 and real China/comparator evidence remain unproven.

## Execution log

| Date | Stage | Result |
|---|---|---|
| 2026-11-22 | Selection | Proposed I335 bounded implementation claim; claim-first #1002 merged before source |
| 2026-11-23 | Implementation | #1003 merged as `bab2bdf31fc8200758397d38077dab57b9dd1aa4`; exact-source CI #45160764389 8/8 SUCCESS and Copilot independent APPROVE; no default/live rights |

## Completion evidence

**Completion Commit: `bab2bdf31fc8200758397d38077dab57b9dd1aa4`**

The existing source [PR #1003](https://github.com/wjhuang88/talos/pull/1003) merged the reviewed HEAD `e255c6d521cdc47377796a4c473a5503ac4586a7`. [CI #45160764389](https://github.com/wjhuang88/talos/actions/runs/45160764389) completed all eight applicable jobs SUCCESS; [independent Copilot protected-review APPROVE](https://github.com/wjhuang88/talos/pull/1003#issuecomment-7103434725) recorded no source/API/Network/security blocker. Latest-main merge CAS was satisfied without unrelated rebase. Status closeout is owner-first and effective when published, not on draft PR creation.

Actual code is a real bounded `SearchTransport::execute_prepared` with both-host authority obtained before send and explicit `[search] backend = "bing-dual"` opt-in; typed source-local WWW→CN fallback only, same proxy/manifest/budget/cancellation, no foreign redirects, forbidden HTTP/CAPTCHA/parser-error rotations, and no changes to default DDG-first/CN-only Auto or public Tool schema. Tests use local fixtures, no external Bing calls. First-attempt intermittent macOS Fork and Windows Actor failures are preserved historically; same-head rerun succeeded. Rollback is opt-in disablement. This completion **does not close** Gate-0 #1000 rights, T3b/#957 live/default activation, D3 #937 or Epic #624.
