# Iteration I332: Amazon Bedrock ConverseStream

> Document status: Complete
> MVP deliverable: controlled Bedrock image/tool Agent rounds over signed HTTP.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Closed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | PROVIDER-008-D Bedrock ConverseStream, destination-bound SigV4, catalog and host integration |
| Claimed At | 2026-11-19 |
| Source Issue | #985 |
| Governance Claim PR | #986 |
| Authorization Mode | Single-maintainer merge |
| Authorization Evidence | User explicitly authorized continuous #724 implementation, testing, independent review, merge and child closure. Distinct human approver unavailable; independent Agent protocol/auth/API review, exact-head CI, validators and CAS remain mandatory. Shared GitHub identity is not independent natural-person approval. |
| Implementation PR | #1009 (merged) |
| Last Updated | 2026-11-23 |
| Handoff / Release Condition | Close only after merged implementation evidence and all published acceptance; retain failures and resume gate. |

## Acceptance and validation

Deliver a stateless Bedrock tool round with ordered images, reasoning where supported, model-specific protocol provenance, usage and explicit terminal/error handling. Test all three tool modes through the public provider factory and permission-gated Agent. Exercise truncated binary frames, CRC errors, fragmented reads, multi-tool association and cancellation. Preserve explicit overrides and legacy configurations. Freeze catalog evidence before enabling any inference. Validate signing with published vectors and capture the final signed request at a controlled endpoint. Document configuration, SDK migration, CLI provenance and release impact in both READMEs and protocol/SDK references. Run locked workspace checks and standard preflight, obtain independent protocol/auth/API review, exact-head CI and merge CAS. #132 retains general credential-chain ownership. Completion Commit: 1745cd3c4d29dd375e680822ffc86f9cc6a3379e.

## Iteration inventory at 19809207

- I249-dependency-upgrade-pilot — retain with current owner.
- I277-desktop-mock-visual-i18n — retain with current owner.
- I302-windows-owned-handle-evidence — retain with current owner.
- I318-native-key-repeat — retain with current owner.
- I324-git-236-preflight-compatibility — retain with current owner.
- I330-arch-035-e2-host-storage-selection — retain with current owner.

I323 closed via #984. Implementation starts after this atomic claim and activation reach main.

I141 and I150 are Complete. At main 21589346, I330 has merged its separate host-binding implementation #979. No Bedrock code is in that slice.

At development head `567a6267`, the locked Bedrock provider filter passed
13 tests, including cancellation during a partial HTTP frame. The standard
local preflight passed governance and workspace check, then stopped at
workspace Clippy on three unwrap_used findings in the frame and response
readers. Workspace tests and SDK fixture checks were not reached by that run.

The next preflight retry passed workspace Clippy and reached workspace tests.
The talos-agent library reported 768 passes and one git_auto failure: the Git
maintenance executable path differed from the bound executable. Running that
same test binary with /usr/local/libexec/git-core first on PATH passed the
isolated case (Git 2.56.1). The complete preflight still requires a fresh run.

Mid-implementation Agent review of 82b66ae7 found that textual projection hid
orphan tool results from validation and that fresh CLI catalog selection could
prompt for an unrelated API key. The first correction is dbed0dcb; 14 Bedrock
provider tests passed, including both textual protocols rejecting orphan results.

At f4c14075 the fresh Bedrock CLI catalog selection regression passed. Standard
preflight stopped in Clippy at the nested unmatched-result check in
bedrock_request.rs (collapsible_match) before workspace tests.

Subsequent local preflight attempts stopped at the talos-mcp git_prepared
case04 Unix socket fixture with PermissionDenied. These attempts did not
establish a passing full preflight. During signed continuation development,
the first focused builds caught a digest formatting error and a fixture API
argument mismatch. After correction, all 15 locked Bedrock library tests
passed, including signed ordinary tool continuation in Native, Compat and
TalosStrict against local HTTP.

## Final acceptance — 2026-11-23

Completion Commit: 1745cd3c4d29dd375e680822ffc86f9cc6a3379e

Implementation #1009 merged after CI4564/run45154117894 passed all eight jobs, including release preflight and native Windows workspace validation, for cc14d769c2c7457f4c49b9faba7fd186225e4074. Independent Agent technical and security review passed on that head. Merge CAS verified current ownership, main compatibility and the reviewed head. Controlled HTTP Agent rounds exercised signed reasoning and permission-gated read_image in Native, Compat and TalosStrict. Local fixture and lint failures remain historical. No live AWS validation, release or general credential-chain completion is claimed.

The additional local all-target Clippy invocation failed on test-code lint findings. The standard workspace Clippy and release preflight passed in CI4564.
