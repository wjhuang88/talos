# STORE-002: Storage And Retrieval Capability Contracts

**Status**: Active / Claimed — I321 A/B proposed, ineffective before governance PR merge

**Type**: Architecture / Contract Convergence

**Source Issue**: [#670](https://github.com/wjhuang88/talos/issues/670)

**Parent / Related**: DATA-002 / #141; SERVER-001 / #142; SERVER-001-C / #361; SERVER-002 / #360; STORE-001; ARCH-035 / #897 (cross-host composition consumer; no duplicate capability extraction).

**Non-overlap note:** STORE-002 exclusively owns the complete storage/retrieval capability taxonomy, domain capability seams and behavior-preserving local implementation adaptation. ARCH-035 / #897 owns how product hosts and `talos-runtime` consume those accepted seams via equivalent composition, preserving existing CLI/TUI/Desktop/SDK behavior. If an interface/adapter is already delivered by STORE-002, ARCH-035 reuses it; it does not reimplement it.

## Objective

Converge Talos's existing persistence, retrieval, and semantic capability boundaries behind explicit contracts without changing current SQLite, FTS5, TLOG, JSONL, schema, transaction, migration, path, ranking, or runtime behavior.

## Scope

Define narrow capability contracts for Log, Relational, Object, Lexical Search, Vector Search, Embedding, and Rerank; preserve domain Stores/Repositories and their invariants; document authority and rebuildability; and provide one explicit local composition that delegates to the existing implementations.

Vector search, embedding, and other unavailable capabilities must have explicit unavailable semantics. Zvec, PostgreSQL, S3, generic ORM/query DSL work, and production behavior changes are out of scope for the first phase.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6 |
| Work Slice | STORE-002 I321 A/B: source-truth authority/capability inventory, stable capability taxonomy and additive typed contract foundation for seven capability families; excludes domain adapter migration, SQL/format changes, host integration, Server and new engines |
| Claimed At | 2026-11-10 |
| Source Issue | #670 |
| Governance Claim PR | #908 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User explicitly selected #670 continuation. Governance-only non-overlapping #908 A/B proposal; no independent approving maintainer is available in its current review record. Single-maintainer claim path requires exact-head CI, both validators, documented authorization reason, no blocking feedback and merge-time CAS; protected implementation API review is still mandatory. |
| Implementation PR | Not started |
| Last Updated | 2026-11-10 |
| Handoff / Release Condition | A/B candidate contracts must pass independent API/architecture review and all applicable tests; later C/D/E/F require separately scoped claims. E2/E3 ARCH-035 waits on accepted functional contract and local delegation. |

## Governance

I321 A/B is a proposed atomic governance claim in #908 and is **ineffective before merge to main**. After merge, only bounded A/B source inventory and additive contract foundation may proceed; C/D/E/F and #897 host implementation remain independently gated. Independent API/architecture review, exact-head locked CI, characterization evidence where applicable, and governance validators are required before B implementation acceptance.

## Acceptance Direction

- Every current persistence/retrieval responsibility has one capability classification.
- Domain invariants remain in domain Stores/Repositories.
- Existing SQLite/TLOG/JSONL implementations delegate through the contracts with zero behavior drift.
- A future talos-server profile and Zvec retrieval implementation can bind without another foundational rewrite.
- The issue remains open until the complete contract layer and evidence are merged.


## Intake history

Initially registered as Intake / Unclaimed on 2026-10-07; narrow I321 A/B claim proposed on 2026-11-10 via #908. Source Issue #670 remains open for C/D/E/F after A/B.
