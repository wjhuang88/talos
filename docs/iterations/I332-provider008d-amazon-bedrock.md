# Iteration I332: Amazon Bedrock ConverseStream

> Document status: Active (proposed upon claim merge)
> MVP deliverable: controlled Bedrock image/tool Agent rounds over signed HTTP.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | PROVIDER-008-D Bedrock ConverseStream, destination-bound SigV4, catalog and host integration |
| Claimed At | 2026-11-19 |
| Source Issue | #985 |
| Governance Claim PR | Pending |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User explicitly authorized continuous #724 implementation, testing, independent review, merge and child closure. Distinct human approver unavailable; independent Agent protocol/auth/API review, exact-head CI, validators and CAS remain mandatory. Shared GitHub identity is not independent natural-person approval. |
| Implementation PR | Not started |
| Last Updated | 2026-11-19 |
| Handoff / Release Condition | Close only after merged implementation evidence and all published acceptance; retain failures and resume gate. |

## Acceptance and validation

Deliver a stateless Bedrock tool round with ordered images, reasoning where supported, model-specific protocol provenance, usage and explicit terminal/error handling. Test all three tool modes through the public provider factory and permission-gated Agent. Exercise truncated binary frames, CRC errors, fragmented reads, multi-tool association and cancellation. Preserve explicit overrides and legacy configurations. Freeze catalog evidence before enabling any inference. Validate signing with published vectors and capture the final signed request at a controlled endpoint. Document configuration, SDK migration, CLI provenance and release impact in both READMEs and protocol/SDK references. Run locked workspace checks and standard preflight, obtain independent protocol/auth/API review, exact-head CI and merge CAS. #132 retains general credential-chain ownership. Completion Commit: Pending.

## Iteration inventory at 19809207

- I141-model-variant-picker-and-status-bar — retain with current owner.
- I150-capability-content-types-persistence — retain with current owner.
- I249-dependency-upgrade-pilot — retain with current owner.
- I277-desktop-mock-visual-i18n — retain with current owner.
- I302-windows-owned-handle-evidence — retain with current owner.
- I318-native-key-repeat — retain with current owner.
- I324-git-236-preflight-compatibility — retain with current owner.
- I330-arch-035-e2-host-storage-selection — retain with current owner.

I323 closed via #984. Implementation starts after this atomic claim and activation reach main.
