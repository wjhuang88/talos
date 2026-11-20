# PROVIDER-008-D: Amazon Bedrock ConverseStream

**Status**: In Progress / Claimed (effective upon governance merge)
**Source Issue**: #985
**Parent Epic**: PROVIDER-008 / #724
**Selected Iteration**: I332

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | PROVIDER-008-D Bedrock ConverseStream, destination-bound SigV4, catalog and host integration |
| Claimed At | 2026-11-19 |
| Source Issue | #985 |
| Governance Claim PR | #986 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User explicitly authorized continuous #724 implementation, testing, independent review, merge and child closure. Distinct human approver unavailable; independent Agent protocol/auth/API review, exact-head CI, validators and CAS remain mandatory. Shared GitHub identity is not independent natural-person approval. |
| Implementation PR | Not started |
| Last Updated | 2026-11-19 |
| Handoff / Release Condition | Close only after merged implementation evidence and all published acceptance; retain failures and resume gate. |

## Acceptance

Deliver a stateless Bedrock tool round with ordered images, reasoning where supported, model-specific protocol provenance, usage and explicit terminal/error handling. Test all three tool modes through the public provider factory and permission-gated Agent. Exercise truncated binary frames, CRC errors, fragmented reads, multi-tool association and cancellation. Preserve explicit overrides and legacy configurations. Freeze catalog evidence before enabling any inference. Validate signing with published vectors and capture the final signed request at a controlled endpoint. Document configuration, SDK migration, CLI provenance and release impact in both READMEs and protocol/SDK references. Run locked workspace checks and standard preflight, obtain independent protocol/auth/API review, exact-head CI and merge CAS. #132 retains general credential-chain ownership. Completion Commit: Pending.
