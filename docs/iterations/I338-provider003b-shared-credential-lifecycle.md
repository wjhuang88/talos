# Iteration I338: Shared provider credential lease lifecycle

> Document status: Active / Claimed
> Published plan date: 2026-11-24
> Baseline: f61b1fa9e6f9c072af86843be8d050c15086bc39
> Planned objective: implement ADR-108 B shared lifecycle without changing existing static or dispatch-credential behavior.
> MVP deliverable: runnable offline synthetic resolver tests prove one refresh, scoped cancellation, bounded lifetime/cache, stale lease CAS and legacy host compatibility.
> Baseline rule: preserve this objective; changed targets require a new iteration ID.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | ChatGPT / GPT-6.2 / PROVIDER-003-B execution session |
| Work Slice | PROVIDER-003-B / #1024: shared credential lease lifecycle, binding, cache, concurrency, revocation, storage/redaction boundary and legacy compatibility; no final-request authorization, replay or vendor acquisition |
| Claimed At | 2026-11-24 |
| Source Issue | #1024 |
| Governance Claim PR | #1025 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer delegated B and parallel governance-only claim preparation; main session owns acceptance. Independent protected-scope technical/security review of exact claim head, both validators, applicable CI and merge CAS required; no approval yet. #1021 applies only to #1020 and is not inherited. |
| Implementation PR | #1034 |
| Last Updated | 2026-11-26 |
| Handoff / Release Condition | No implementation before claim merge; hand C the reviewed lifecycle contract and source/test evidence; maintain #132 open and close B only with existing implementation SHA and independently reviewed acceptance. |

## Published Baseline

| Story | Parent | Status At Selection | Depends On | Outcome |
|---|---|---|---|---|
| [PROVIDER-003-B](../backlog/active/PROVIDER-003-B-shared-credential-lifecycle.md) / #1024 | PROVIDER-003 / #132 | Ready for proposed activation subject to model/limits review, governance checks and independent approval | I336 / #1013 Complete; ADR-108 Accepted | Shared lifecycle + C handoff + compatibility fixtures |

Scope, detailed acceptance, numeric candidate bounds, source evidence, model results and residuals are the linked B owner. Acceptance includes binding, generation/revision separation, one producer per key, per-waiter cancel, logout/replacement/shutdown, L1 late-401 after L2, cache/clock limits, diagnostics, no dynamic file fallback and legacy CLI/Desktop/SDK. No final authorization, HTTP replay, vendor acquisition, OS-store implementation or product-code change in this claim PR. C owns transport integration; D–G separate.

## Selection inventory and disposition

Read AGENTS, collaboration and START-ITERATION SOP, parent, ADR-108, iteration template, roadmap, Board and iteration index at exact base. Directory listing highest I337; search found no existing B owner or issue other than parent historic references, formal parent sub_issues endpoint returned [] before #1024, no B or I338 branch match at selection; #1024 linked logically via parent references (no native sub-issue mutation used). Open PR inventory rechecked before finalization: #1023 Agent checkpoint diagnostic fixture, #1019 I324 Git D1 provenance, #1018 I337 MCP fixture, #1010 I334 session startup fixture, #997 Desktop closeout; none owns B credential resolution. Shared host paths are compatibility observations only; no I334 composition implementation. A #1013/I336 Complete/Closed #1022; #724 all four terminal and no reimplementation. I338 chosen as next observed free ID. Recheck current main/ID/branches/PRs and scope at merge CAS.

| Iteration or gate | Disposition |
|---|---|
| I324 Active / Claimed, #919/#1019 | Retain active older-Git scope; native acceptance outstanding, no B overlap. |
| I334 Active / Claimed, #995/#1010 | Retain active CLI/SDK Session storage composition/fixture, independent; no Host migration in B. |
| I337 Active / Claimed, merged #1016 / #1018 | Retain offline synthetic MCP work; no supplier admission or credential sharing with B. |
| I302 Review / Claimed | Retain broader native Windows handle residual #717; I305 narrow acceptance closed. |
| I318 Review / Claimed | Retain physical TUI terminal/IME acceptance #269/#872. |
| I277 Review / Claimed | Retain deferred human/device validation #29. |
| I249 Planned / Unclaimed | Defer unchanged dependency-upgrade pilot; no dependency upgrade in B. |
| I164 Paused | Preserve superseded layout target. |
| I329 T3a Complete / Closed; T3b Blocked / Unclaimed | Keep #957 external rights/region gate blocked. SEARCH-001 E/F/G/I and #848 admission separate; no live search work. |
| I300 Complete / Closed | Latest owner T7 closure ee84889f supersedes old index Active text. |
| I330, I331, I332, I333, I335, I336 | Terminal; do not repeat. |

Board also records independent Review/Active tasks (e.g. DEPENDENCY-003-A, clipboard, git snapshot, render performance, history/UI and Desktop closeout) and blocked ARCH-035 E3b/E4, STORE-002 F, ServerHost, permission and SEARCH admission gates; retain their owner-defined dispositions. No claim/activation transferred. Inventory derives from full iteration directory names, index/Board and direct reads of the named nonterminal owners; not a full local scan of all 334 iteration bodies due checkout DNS limitation. Independent review/merge CAS must complete any missing exhaustive inventory and verify no new overlap; do not misstate a fully validated inventory. Maintainer explicitly delegated parallel nonoverlapping B, main session responsible for acceptance.

## Planned Validation

- Prior to activation: model described in B owner ran 16/16 named assertions; previous exploratory mislabeled count retained there. No Rust/runtime proof. Candidate numeric boundaries subject to main-session and independent review; both governance scripts and exact-head scoped CI including Issue reconciliation required.
- After effective merge: deterministic fake-source/counted single-flight, barrier cancellation and stale publish tests, offline explicit-source conflict rejection (redacted error, zero acquisition/store/helper calls and outbound requests; omitted-auth precedence positive controls), cancellation-delayed fake with actual running/peak counters proving <=16 across repeated detach/reacquire, permit recovery only after confirmed exit, injected clocks/capacity/CAS, synthetic static config/host matrix and canary redaction; cargo check --workspace --locked; cargo clippy --workspace --locked -- -D warnings; cargo test --workspace --locked; applicable native Linux/macOS/Windows CI and protected-scope/API review. No live vendor data. Preserve skips and failures.
- Documentation: B owner, C handoff, user-facing README / config reference and SDK contract as applicable; parent/Board/index/Issue owner-first sync. No new configuration in governance-only PR.
- Risks and rollback: cross-profile leakage, cancellation races, time skew, secret retention and static drift; private resolver and controlled sources, no migration of vendor implementations until independently claimed, preserve static behavior and fail closed.

## Actual Activation And Execution

| Date | Type | Record |
|---|---|---|
| 2026-11-24 | Proposed atomic claim + activation | #1025, source #1024; both become effective ONLY after target-main merge. No implementation started. |
| 2026-11-24 | Read-only evidence | Base source and 16/16 arithmetic/CAS model; local clone blocked by DNS, exact-head CI and both validators not yet verified. |
| 2026-11-24 | Authorization | Independent review required. #1021 single-maintainer exception only #1020. Main session must resolve blocking review, confirm numeric/model gates, check exact head, governance validators, remote reconciliation and repeat CAS; no merge by executing session. |

| 2026-11-24 | Independent review revision | Review of 0339189f requested explicit-source admission and actual producer lifetime bounds; B owner and planned validation revised only. CI4620 passed on reviewed historical head; new exact-head CI and re-review required. |

## Verification Evidence / Residuals

Governance CI receipts and review to be appended as PR comments bound to exact head; no fabricated local validator result. Product tests and actual shared resolver unimplemented. Remaining C–G acceptance separate; #132 and #1024 open. No Completion Commit.

### Implementation checkpoint (2026-11-24)

Activated by #1025 merge `2645018339892020b7d01633b8a6f9ad6615c197`. First library source checkpoint on `feat/i338-provider003b-credential-lifecycle`: resolver and deterministic fake-source fixtures. Main session may assist local pinned-toolchain validation; no compile/test result yet. Continue with local formatting, focused tests and review, remaining compatibility and boundary fixtures, then stable implementation PR and exact-head CI. No product enablement or completion.

### First local validation feedback and correction (2026-11-24)

Previous head `2b7a3a6ada1e3deab82b4dcbdd69b198ae8454f4`: pinned 1.99.0 fmt check exit 1, check/test exit 101 (E0308), focused tests executed **0**, Clippy exit 101 including seven pre-existing Azure diagnostics, itemized in B owner and #1024 comment 7134583357. Fix revoked-write fixture argument and isolate synchronous RefreshStore I/O from global registry lock via bounded per-profile reservations and post-I/O generation checks. Added controlled slow-read/write and reentrant store fixtures; admitted writes may complete physically but revoked results cannot succeed; durable store cleanup is external enrollment scope. Local rustfmt unavailable here; main-session formatting and four-command retest required. Do not mark validated, Review or Complete on source alone.


### Second local validation and acceptance-gap checkpoint

Maintainer reports local formatted SHA `5113d175fd41c715943846bb98f89c7d5ddc495c`: pinned fmt/check passed, provider focused **12/12 passed**, strict all-targets Clippy exit 101 (95 diagnostics reported in unchanged Azure/Bedrock files). Historical E0308/0-test receipt retained. Remote format-only sync `80fe048b02633cb2f7aa44bc92b54671aa2304cc` verified as child of 753abfb9; local SHA object unavailable remotely, no claim of same exact SHA verification. See B owner acceptance mapping for named tests and remaining gates. Main moved to 262b6ca7 (I324-only changes, no overlap); retain implementation history.

New source checkpoint closes identified implementation/fixture gaps: trusted binding inventory and atomic replacement (64 profiles/256 enrolled specs, subject to review); explicit CredentialContext with shared handles, shutdown and drop cancellation; waiter limits, deterministic timeout, clock regression, acquisition failure and enrollment isolation fixtures; shared factory and CLI/Desktop/SDK static compatibility previews. 20 provider focused test definitions plus 3 host fixtures, not yet executed on this candidate. SDK contract updated; no default runtime or vendor auth change. Need pinned rustfmt, locked focused checks/tests/Clippy, config and host regression suites, workspace/native CI, stable implementation PR with exact-head evidence and independent protected-scope review. Do not mark Complete or close #1024; C and D–G separate.

## Phase three local verification and implementation PR (2026-11-26)

Claim #1025 effective on main at 2645018339892020b7d01633b8a6f9ad6615c197. Implementation PR #1034, independent review required; no completion or merge authorization. Main-session receipt at [#1024 comment 7167456663](https://github.com/wjhuang88/talos/issues/1024#issuecomment-7167456663), tested code SHA `d7102b28d6aa71af39cebf743ffb708554d73dec`, pinned Rust 1.99.0: fmt/check passed; provider credential_lifecycle 20/20; config 309/309; CLI and SDK 2/2. Maintainer supplied the follow-up Desktop output for the requested explicit desktop-ui test: `provider::tests::credential_lifecycle_legacy_static_desktop_preview ... ok`, 1 passed, 0 failed, 0 ignored, 117 filtered out. This is user-reported local execution, not a CI receipt. Strict all-targets Clippy exit 101 with 95 reported existing Azure/Bedrock diagnostics; earlier E0308 and zero executed tests retained. All named provider fixtures in the preceding mapping are covered by the reported 20/20 run; the three static host fixtures and config suite passed as reported. No new product or test changes in this receipt commit. Exact-head CI, workspace/platform results, independent lifecycle/security/API review and merge-time CAS remain outstanding. Main checked at a40a077d23c8d28803844da1bfff90f011ec96d7; activation is its ancestor, no changed-file overlap with B in the comparison, no history rewrite. Keep B In Progress/Active until review gates are satisfied; C/D–G separate, parent and child remain open.
