# Iteration I261: Rust WASM Language Provider Vertical Slice

> Document status: Planned / Claimed (pending governance merge)
> Parent: CAP-001 / #466; LANG-002 / #516
> Objective: prove one manually installed Rust WASM Language Provider serves highlighting and symbol consumers through the shared contract.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex unattended single-developer mode |
| Work Slice | One Rust WASM LanguageProvider vertical slice through verified Bundle installation, bounded loading, registration, and shared consumers. |
| Source Issue | #516 |
| Claimed At | 2026-09-12 |
| Governance Claim PR | #542 |
| Implementation PR | Not started |
| Authorization Evidence | Proposed claim is ineffective until merged to main; implementation remains unauthorized. |
| Authorization Mode | Independent review |
| Last Updated | 2026-09-12 |
| Handoff / Release Condition | Claim #542 must merge before implementation; independent security/dependency review required. |

Claim preparation is governance-only; implementation starts only after the claim reaches main.

### Non-terminal iteration inventory — 2026-09-12

| Item | Current disposition |
|---|---|
| I261 / LANG-002 | Planned / Claimed; claim pending merge, implementation unauthorized |
| I164 / legacy migration | Paused / superseded; do not resume |
| I249 and other active capability work | Frozen or separately governed; no overlap with this slice |
| LANG-003 / #517, DIST-001-B / #515, BROWSER-001 / #508 | Refinement / Unclaimed; remain future work |
| I159–I162 | Terminal or separately blocked; unchanged |
| Open implementation PRs | None claimed by I261; unrelated PRs remain outside this slice |

This inventory is a governance checkpoint, not implementation authorization.

### Claim Preparation Checkpoint — 2026-09-12

Owner and work slice are recorded locally for governance review. This checkpoint does not
authorize implementation; the claim becomes effective only when its governance PR merges to main.

## Scope and Acceptance

- Resolve one `language.rust` WASM Provider through the shared contract for TUI highlighting and symbol queries.
- Use a manually verified Bundle; enforce loader limits, trap/resource containment, timeout handling, and plain-code fallback.
- Keep parser-native types behind the provider boundary and preserve no-network startup behavior.

## Non-Goals

No remaining-language migration, marketplace/download flow, Desktop binding, or silent executable acquisition.

## Dependencies

LANG-001/I257, CAP-001-C/I255, BUNDLE-001/I259, and DIST-001-A/I260 are Complete / Closed.

## Governance Gate

Before implementation, assign an owner, establish an effective Collaboration Claim, and obtain an independent security/dependency review. The claim is currently Claimed but remains ineffective until PR #542 merges to `main`; until then this iteration remains Planned and implementation is unauthorized.
