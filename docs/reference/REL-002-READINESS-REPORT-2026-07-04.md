# REL-002 Readiness Report

**Date**: 2026-07-04
**Plan item**: I093/A13
**Gate**: REL-002 v1.0 self-bootstrap release gate
**Verdict**: Not ready for `v1.0.0`

## Executive Summary

REL-002 remains unsatisfied. The 2026-07-04 direct-owner track improved several high-risk
prerequisites: model catalog startup behavior was clarified earlier, ingestion/search bounds were
hardened, plugin/hook/distribution boundaries were narrowed, bash compression got cache/export
regressions, and autonomy permission gates now have a deny/ask/allow matrix. None of that is
Talos-primary self-bootstrap evidence.

This report updates the readiness picture for `RUNTIME-001`, `GOV-003`, and `ARCH-030`. It does
not authorize a tag, publish, GitHub Release, or `v1.0.0` claim.

## Evidence Reviewed

| Evidence | Result |
|---|---|
| `RUNTIME-001` owner doc and `talos-runtime` status | Pre-1.0 facade is complete and useful, but the self-bootstrap loop still lacks Talos-primary editing, validation execution, git/issue sync, and release authority boundaries. |
| `GOV-003` owner doc | Phase 1 read-only status exists (`--governance-status`, `/agile status`, dashboard route). Mutating governance, gate enforcement, initialization/adoption, and repair remain future work. |
| `ARCH-030` residual register | Residual roots are explicitly owned, but no readiness audit has converted them into release-blocking/non-blocking decisions for self-bootstrap. |
| I090-I092 direct-owner work | Improves safety prerequisites, but Codex remains the primary executor; therefore it is non-qualifying for REL-002. |
| Previous readiness report | `docs/reference/REL-002-READINESS-REPORT-2026-07-02.md` remains accurate: no qualifying Talos-primary sessions exist. |
| I095 runtime validation evidence update | `talos validate run` now executes built-in allowlisted profiles and records command, exit status, stdout/stderr summaries, and the allowlisted-profile permission decision. This narrows the validation evidence gap but is not Talos-primary self-bootstrap evidence by itself. |

## Acceptance Assessment

| REL-002 Acceptance Item | Status | 2026-07-04 Assessment |
|---|---|---|
| Talos can run a complete development iteration as primary runtime. | Not met | Current I090-I093 work is Codex-primary. |
| Talos can read governance state, select work, and preserve owner docs. | Partial | Read-only governance views exist; owner-doc mutation/sync still depends on external execution. |
| Talos can implement code changes with permission-gated tools and produce validation evidence. | Not met | Runtime tools exist, but no qualifying Talos-primary implementation session is recorded. |
| Talos can perform architecture-risk classification and route high-risk work. | Partial | ARCH-030 and governance docs classify risks, but Talos does not yet own an executable risk-classification workflow. |
| Talos can update README/user docs/backlog/iterations/decisions/board after real change. | Not met | Updates are still performed by Codex in current evidence. |
| At least three non-trivial Talos-on-Talos development sessions are recorded. | Not met | Existing rehearsals are non-qualifying; I090-I093 are also Codex-primary. |
| Codex is not the primary executor for qualifying sessions. | Not met | Codex remains primary. |
| Final v1.0 release checklist names all remaining gaps. | Partial | This report updates gaps; final checklist remains future work. |

## Runtime SDK Gaps

| Gap | Current State | Minimum Next Evidence |
|---|---|---|
| Talos-primary edit loop | `talos-runtime` can submit turns and stream events, but no Talos-primary repo edit session qualifies. | A documentation-only Talos-primary session where Talos plans, edits, validates, and records evidence with Codex limited to review. |
| Validation execution | I095 adds `talos validate run` for built-in allowlisted profiles with durable command, output-summary, exit-status, and permission-decision records. | Use the evidence packet in a Talos-primary session; it is not enough while Codex remains the primary executor. |
| Git/issue sync | Git push/issue comment remains external. | Explicit policy: either Talos gains permission-gated git/issue workflow or REL-002 accepts external release-operator boundary. |
| SDK stability | `RUNTIME-001` is complete as pre-1.0 facade. | Decide which `talos-runtime` APIs graduate to stable support and which remain internal before v1.0. |

## Governance Gaps

| Gap | Current State | Minimum Next Evidence |
|---|---|---|
| Read-only governance state | Shipped via CLI/TUI/dashboard status paths. | Keep; verify it reads current I090-I093 state without drift. |
| Mutating governance actions | Not implemented. | Typed plan/preview/write flow that uses permission-gated file edits and validates owner docs after mutation. |
| Gate enforcement | Not implemented. | Read-only violation detection first, then explicit enforcement policy; no silent blocking without user-visible reason. |
| Risk classification | Mostly prompt/docs-driven. | Deterministic report that routes changes to ADR/security/permission/release gates before coding. |

## Architecture Residual Gaps

ARCH-030 remains useful as a watchlist. For REL-002, each residual root needs a release posture:

| Root | Current REL-002 Risk | Next Action |
|---|---|---|
| `crates/talos-cli/src/mode_runners.rs` | Medium. Large orchestration root can hide mode/session drift. | Audit only when next CLI lifecycle feature touches it. |
| `crates/talos-tui/src/app.rs` | Medium. Visual/input state is large and regression-prone. | Keep screenshot/state tests for any frame/input work. |
| `crates/talos-session/src/sqlite.rs` | High. Session history/search/fork storage affects self-bootstrap continuity. | Before v1.0, split or audit schema/search/fork paths with migration rollback evidence. |
| `crates/talos-tools/src/git.rs` | Medium-high. Read/write Git behavior and host fallback share one root. | Separate read-only and write-capable Git paths before Talos-primary git workflow expands. |
| Provider roots | Medium. Stream parsing/retry/tool-call handling affects reliability. | Split only when next provider protocol work touches them. |
| Exploration roots | Low for immediate REL-002 unless research library becomes self-bootstrap-critical. | Keep watchlist until exploration is in the primary loop. |

## Go / No-Go

| Decision | Result |
|---|---|
| `v1.0.0` self-bootstrap release | No-go |
| Claim "Talos can perform 100% self-bootstrap development" | No-go |
| Continue pre-1.0 hardening | Go, with explicit pre-1.0 posture |
| Real tag, publish, or GitHub Release | No-go without separate maintainer approval |

## Minimum Next Packet

The next highest-value REL-002 packet is not another broad feature. It is a controlled
Talos-primary rehearsal:

1. Select a documentation-only or read-only code audit change.
2. Run it through Talos as the primary runtime.
3. Require Talos to produce the edit, validation evidence, owner-doc sync, and residual report.
4. Label any Codex intervention as review/fallback.
5. Record whether the session qualifies.

## Conclusion

Talos remains pre-1.0. The runtime facade, governance views, and architecture residual register are
better prerequisites than they were, but REL-002 needs real Talos-primary sessions before any
`v1.0.0` claim is technically honest.
