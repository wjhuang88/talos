# Iteration I278: Repository-Root Plugin Source Delivery

> Document status: Active
> Published plan date: 2026-09-18
> Planned objective: Deliver the omitted repository-root plugins/ source and independent build/package path from #466.
> Baseline rule: preserve this objective and acceptance; append execution facts.
> MVP deliverable: build an optional Plugin/Provider from root plugins/ source, package/install it as a verified Bundle and execute its real input-dependent capability.

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Claimed |
| Responsible Actor | @wjhuang88 |
| Executing Agent | Codex / GPT-6 |
| Work Slice | CAP-001-D real root Rust/Python Plugin sources, independent build/package, compatible bounded language transport and real-consumer validation under ADR-079; no Desktop or default-distribution change |
| Claimed At | 2026-09-18 |
| Source Issue | #466 |
| Governance Claim PR | #575 |
| Authorization Mode | Independent review |
| Authorization Evidence | Maintainer I278 delivery goal and delegated Agent-role review; #575 exact-head security/API/governance approval and scoped CI required before merge; activation effective only on main |
| Implementation PR | Not started |
| Last Updated | 2026-09-18 |
| Handoff / Release Condition | #575 target-branch merge before implementation; ADR-079 boundary and all baseline acceptance mandatory; I277/#29 human/device rows remain Deferred, non-blocking and unverified |

## Published Baseline

Selected Story: [CAP-001-D](../backlog/active/CAP-001-D-root-plugin-source-layout.md).
Dependencies: existing CAP-001-B/C, language contracts, Bundle/manual installation and
ADR-027/072/073. Verify current target-branch dependency evidence and effective claim before code.

Scope: inventory concrete providers versus fixtures; implement root source organization and
independent build/package entrypoints; exercise installed artifacts; update affected consumers,
tests and layout documentation. Host infrastructure remains in `crates/talos-plugin/`.

Acceptance: every CAP-001-D acceptance item is mandatory. Empty directories, source relocation
without executable packaging, and canned fixture results cannot establish completion. Prove
input-dependent capability behavior, safe failure, dependency isolation and reference consistency.

Non-goals: Desktop, new Browser capabilities, marketplace, implicit activation/acquisition,
default distribution changes, release or publication. Preserve published prior iteration evidence.

Validation: focused build/package/install/consumer and failure tests; locked preflight for code;
governance validators and whitespace checks; stable-candidate CI and appropriate independent review.
Documentation: Plugin build/package/usage guide, host README, architecture layout, ADR-072,
Story/iteration and owner-first derived indexes.

Risk: existing artifacts may prove only a test seam, not distributable implementation. Inventory
this before moving files and implement the missing bounded delivery rather than relabel fixtures.
Rollback: revert the source/build migration without changing installation identity or host policy.

## Scheduling And Inventory — 2026-09-18

The maintainer explicitly prioritizes this omitted requirement for the next iteration cycle.
The local iteration-header inventory found I277 Review and I249 Planned, with no Active or
Blocked current header. I164 remains Paused under its superseded layout target. Completed-owner
historical Review checkpoints are not new active claims.

| Owner | Disposition |
|---|---|
| I277 | Continue current Desktop corrections and machine/review/merge gates first. Retain deferred human validation in #29; do not fabricate Complete. |
| I249 | Keep published dependency pilot Planned/Unclaimed; not selected ahead of I278. |
| I164 | Keep Paused; no reactivation or layout work authorized. |
| I278 | Planned/Unclaimed, first next implementation cycle; establish target-branch claim at handoff. |

No plugin implementation begins in the I277 branch. Keep this planning change separate from the
Desktop implementation commit/candidate. Reuse #466 as source; no per-subtask Issue is required.

Evidence correction: I253's R1 `Satisfied` label explicitly excluded source delivery. Its cited
ADR-072/architecture layout contract is not explicit in the current files. This plan owns both
the missing contract clarification and implementation; the historical label is not acceptance.

## 2026-09-18 Source And ABI Investigation

Requested outcome: complete CAP-001-D source/build/package/install/real-consumer delivery.
Preserve the Published Baseline, existing legacy fixtures, I277 deferral evidence and unrelated
owners. Synchronize I278/CAP-001-D first, then parent/manifest/Board/indexes. Required evidence
remains focused real-artifact tests, locked preflight, independent security/API review and CI.
Any unresolved delivery gap remains owned here rather than creating per-subtask Issues.

At `main@d6b566be`, no open PR overlaps this slice. I277 implementation #571/#573 is merged;
remaining human/device acceptance is explicitly Deferred by the maintainer and non-blocking.
I249 stays Planned/Unclaimed and I164 Paused. I278 remains Planned/Unclaimed pending its atomic
claim and decision review; read-only investigation grants no implementation authority.

| Existing artifact | Verified behavior | Disposition |
|---|---|---|
| `crates/talos-plugin/tests/fixtures/language-provider/provider.wat` | Constant zero response, plain-text fallback | Keep as a negative/empty-response compatibility fixture; not production Rust capability. |
| `crates/talos-plugin/tests/fixtures/language-provider-python/provider.wat` | Same constant response | Keep as an installation/lifecycle fixture; not production Python capability. |
| Inline WAT response fixtures in `crates/talos-plugin/src/wasm.rs` | Canned protocol responses | Preserve decoder tests; add separate real-artifact consumer evidence. |
| Built-in `talos-text` language implementation | Actual statically linked source processing | Preserve current defaults; no broad parser migration. |
| `crates/talos-plugin` | Host load/install/lifecycle/transport infrastructure | Keep in `crates/`; concrete optional sources belong under root `plugins/`. |

The legacy host writes at guest address zero and lacks store memory/table limits in the language
execution path; response copying also precedes its size check. These are verified integration gaps,
not evidence of a deployed escape. Real Rust guest delivery requires an explicit buffer contract
and bounded execution before claiming safety. [ADR-079](../decisions/079-rust-language-plugin-guest-buffer-boundary.md)
proposes an additive allocator handshake and only guest symbol-export unsafe attributes; no raw
pointer memory operations. Independent security/API review is required before acceptance.

Actual consumer evidence must use the shared TUI highlight and symbol-tool paths. The current print
registration path discards the language context, so it is not existing language-consumer evidence.
The implementation must account for every affected path without presenting fixtures as real work.

### Decision Review And Claim Preparation

Independent Agent-role security/API review of ADR-079 initially requested two changes: bound
admission/version probing as well as execution, and reject new guests on old hosts before any
request memory write. The corrected proposal adds ABI v2 with mandatory allocation, retains
new-host/v1 support, and explicitly covers public probe callers and artifact start/version rules.
Reviewer `/root/i278_abi_decision_review` issued APPROVE against base `d6b566be25205a5abe1202dabeb0cee09ea81cce`
and ADR content blob `1a6182b425741a0f6da0825b69c4ab8595241e72`. Shared workspace/account;
Agent-role separation only. This is decision evidence, not implementation safety acceptance.
The draft governance candidate will bind the actual claim PR before becoming reviewable.

### Atomic Claim Proposal — PR #575

PR #575 proposes Active / Claimed and ADR-079 acceptance together. Neither is effective until
the finalized candidate reaches `main` after exact-head CI, independent security/API/governance
approval and merge-time CAS. Implementation must start from that merge or a later main commit.
The maintainer's standing single-maintainer/unattended instruction delegates independent review
to a separate Agent role; it does not waive protected review or claim a separate human identity.

I277's #571/#573 machine/review/merge gates have passed; its remaining manual/device checks are
Deferred and stay with I277/#29. I278 does not modify Desktop code, its runtime binding or its
acceptance baseline. I249 remains Planned/Unclaimed; I164 remains Paused. Existing I277 local/
remote branches are predecessor evidence, not competing Plugin claims. The PR list was empty
before opening #575. Source #466 remains the closed architecture parent; CAP-001-D owns this
recovered implementation requirement without reopening or broadening historical completion.

Expected implementation inventory: root `plugins/` sources/build/package guide and lockfile;
bounded language transport and tests in `crates/talos-plugin`; additive protocol support in
`crates/talos-text`; real consumer integration tests in TUI/tools/runtime/CLI only as needed;
scoped build/CI wiring; ADR-072/architecture/layout and owner evidence. Runtime default dependency
expansion, native loading, permission policy, Desktop and new Browser capability are excluded.
Any newly discovered incompatible public API or unsafe-memory need requires decision review,
not an implementation-time exception. Local convergence precedes the stable implementation PR.
