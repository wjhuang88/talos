# BROWSER-001: Read-Only Browser Provider Connector

| Field | Value |
|---|---|
| Story ID | BROWSER-001 |
| Type | Browser capability implementation |
| Parent | CAP-001 / #466; WEB-005 |
| Status | Refinement / Unclaimed |
| Selected Iteration | None |
| Source Issue | [GitHub Issue #508](https://github.com/wjhuang88/talos/issues/508) |
| Depends On | CAP-001-B/C; WEB-005 security/product semantics; TOOL-014 disclosure policy |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | One read-only Browser Provider connector behind the existing browser-page boundary. |
| Claimed At | Not applicable |
| Authorization Evidence | No effective claim; intake owner only. Implementation is not authorized. |
| Governance Claim PR | Not applicable |
| Implementation PR | Not started |
| Authorization Mode | Not applicable |
| Last Updated | 2026-09-09 |
| Handoff / Release Condition | Requires a connector ADR/security review and explicit overlap agreement with WEB-005/TOOL-014. |

## Required Reads

- [CAP-001 parent](CAP-001-progressive-capability-provider-architecture.md), [ADR-072](../../decisions/072-capability-provider-bundle-boundary.md), and [WEB-005](WEB-005-browser-session-continuity-research.md).
- [TOOL-014](TOOL-014-conditional-tool-backends.md) and the connector ADR/security gate. This read-only connector does not authorize frame-aware interaction requested by Issue #520.

## Goal And Scope

Implement a selected read-only browser connector that produces the existing redacted
BrowserPageRecord semantics through the Capability/Provider boundary.

## Non-Goals

No click, fill, upload, download, checkout, cookie/storage/credential extraction, Desktop UI or
alternate permission authority.

## Acceptance

- Connector discovery/loading is separate from TOOL-014 disclosure and permission evaluation.
- Origin, TTL, revisit, redaction and size limits follow WEB-005 decisions.
- Connector, network, browser and provider failures degrade to explicit unavailable/error results.
- No browser activity starts at startup without an explicit request.

## Validation And Documentation

Read-only connector fixtures, redaction/security tests, permission/disclosure integration checks,
network failure/cancellation tests and user-facing browser capability documentation.
