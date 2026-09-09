# LANG-002: Rust Language Provider Vertical Slice

| Field | Value |
|---|---|
| Story ID | LANG-002 |
| Type | Language Provider implementation |
| Parent | CAP-001 / #466 |
| Status | Refinement / Unclaimed |
| Selected Iteration | None |
| Source Issue | [GitHub Issue #516](https://github.com/wjhuang88/talos/issues/516) |
| Depends On | LANG-001; CAP-001-C |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | One Rust LanguageProvider vertical slice through the approved contract. |
| Claimed At | Not applicable |
| Authorization Evidence | No effective claim; intake owner only. Implementation is not authorized. |
| Governance Claim PR | Not applicable |
| Implementation PR | Not started |
| Authorization Mode | Not applicable |
| Last Updated | 2026-09-09 |
| Handoff / Release Condition | Requires LANG-001 implementation evidence and a separate security/dependency review. |

## Goal And Scope

Prove one end-to-end `language.rust` Provider can serve highlighting and symbol consumers through
the shared contract, with explicit provider-unavailable behavior.

## Non-Goals

No remaining-language migration, marketplace/download flow, broad parser trimming, Desktop binding,
or silent executable acquisition.

## Acceptance

- Rust code resolves one Provider and both TUI and symbol paths consume it.
- Missing, corrupt, incompatible or timed-out Provider leaves the process healthy.
- No parser-native types cross the shared boundary and no default behavior regresses.

## Validation And Documentation

Offline end-to-end fixtures, failure-path tests, dependency/size evidence, locked checks and
security review. Binary-size reduction is not claimed unless measured by this slice.
