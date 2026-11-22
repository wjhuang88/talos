# I335: Bing Dual Host Real Transport + Explicit Opt-in (SEARCH-001-D3-T5)

> Document status: Proposed / Unclaimed, not effective before atomic governance merge.
> Selection: 2026-11-22. Maintainer prioritizes working functionality over supplier-rights research.
> Goal: Complete an executable, secure, permission-aware WWW-first/CN-fallback search path without changing the default network route or equating implementation with licensing.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | I335 / D3-T5 #1001: real transport, explicit opt-in, deterministic offline transport fixtures; no default/live admission |
| Claimed At | Pending |
| Source Issue | #1001 |
| Governance Claim PR | Pending |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | Maintainer directs implementation 2026-11-22; independent API/Network/security review and exact-head CI/CAS required |
| Implementation PR | Not started |
| Last Updated | 2026-11-22 |
| Handoff / Release Condition | Effective claim before implementation; source review/CI/owner-first closeout after; supplier rights Gate-0/#1000 separate |

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
| 2026-11-22 | Selection | Proposed I335 bounded implementation claim; source blocked until governance PR merged |

## Completion evidence

None. Not Complete until separately merged source and owner-first closeout.
