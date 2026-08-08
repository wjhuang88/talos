# Iterations

## Purpose

Track current iteration plans, execution state, verification evidence, and retrospectives. Each
iteration's own document is authoritative for its scope and lifecycle.

The complete pre-closeout index is preserved unchanged at
[`ITERATIONS-INDEX-pre-I170-closeout-2026-08-01.md`](ITERATIONS-INDEX-pre-I170-closeout-2026-08-01.md).
That snapshot is historical evidence and not current activation authority.

## Lifecycle

1. **Planned** — objective, selected stories, acceptance and activation gate are published.
2. **Active** — explicitly activated work is in progress on a fresh current-main branch.
3. **Review** — implementation exists and required evidence/review is pending.
4. **Complete** — verification, acceptance, completion commit and retrospective are recorded.
5. **Paused / Blocked / Superseded** — non-active states with an explicit resume or replacement gate.

## Rules

- Every iteration has a unique ID.
- Published baselines are not silently repurposed; changed objectives use a new ID.
- Before activation, inventory current Issues, PRs, branches, owner docs and other non-terminal work.
- Ready/Planned does not mean Active.
- Recovery branches and PRs are provenance only unless a new current-main plan explicitly says otherwise.
- Complete requires runtime/acceptance evidence appropriate to the scope, not unit tests alone.

## Current Operating Set

I158 and I171 completion evidence: `Completion Commit: c88c1d1a` (existing implementation/audit
closeout evidence; status synchronization commits do not self-certify completion).

| ID | Codename | State | Activation / Completion Gate |
|---|---|---|---|
| I158 | Tool Registration Composition Consolidation | Complete | Completion Commit `c88c1d1a`; scheduler/status exceptions and documentation closeout accepted. TUI-037 remains independent. |
| I171 | Workspace Architecture Rebaseline | Complete | Completion Commit `c88c1d1a`; v0.7.0 audit/register and bounded remediation owners validated; no production refactor. |
| I159 | `talos-tools` Lightweight Feature Boundary | Blocked | Requires the completed I158 baseline and a recorded TUI-037 disposition before activation. |
| I160 | Shared CLI And Runtime Internal Composition | Blocked | Requires I159 Complete. |
| I161 | Sandbox Fallback And Coding Preset | Blocked | Requires I160 Complete and an independent security-review plan. |
| I162 | v0.6 SDK Fixture And Publication Readiness | Blocked | Requires I161 Complete and explicit readiness authorization; no real publish/tag/release. |
| I172 | CLI/TUI Bridge Legacy Projection Decomposition | Complete | Completion Commit `4084138dc0652d3200045847d42518d9ecb66231`; PR #144 merged at `c1dc67ae`; exact-head CI `31137882248` passed. |
| I173 | Todo Module Decomposition | Complete | Completion Commit `e4818e34c1e047c41d41abc1f7859c7984008e83`; PR #149 merged as `506311dc`; exact-head CI `31143057387` passed. |
| I174 | TUI App Coordinator Decomposition | Complete | Completion Commit `e4248bfedd17c91aebb24c80c60580fcbcebec62`; PR #152 merged at `62b09c277713bea8404ed7ef9c7f50354e5a2e17`; exact-head CI `31148908291` passed. |
| I175 | Conversation Engine Decomposition | Complete | Completion Commit `5c45322245788e12316dffbe1f9cfacef390eff8`; PR #156 merged at `73898bdba0d072886c79023c048250190a3b5e04`; exact-head CI `31152972959` passed. R04 remains Refinement; later child status is tracked by its own row. |
| I176 | CLI Session Handler Decomposition | Complete | Completion Commit `1de3243d`; PR #159 merged at `37c557271b906664022476bd2775c5cd77f2b8ea`; exact-head CI `31160309818` passed. R04 remains Refinement; later child status is tracked by its own row. |
| I177 | Agent Session Custody Decomposition | Complete | Completion Commit `f505eea8` (squash merge of implementation `786aa571`); PR #162; exact-head CI `31166594367` passed. |
| I178 | Pending Submission Store Decomposition | Complete | Completion Commit `f92634803560dc50e0b15ca8d7d511e9928c983f` (squash merge of source implementation `c662a7e6`); PR #165; exact-head CI `31180591881` passed. |
| I179 | Core Tool Facade Decomposition | Complete | Completion Commit `dafc9be08736aee91e0f9cdd92e5226930808061` (squash merge of source implementation `63d494c5`); PR #168; exact-head CI `31189425069` passed. |
| I180 | Architecture Documentation Truth | Complete | Completion Commit `10cceec6aeb9089fe9c830355992c8fc60430d63` (squash merge of source implementation `fd8ac75d`); PR #171; exact-head CI `31238721507` passed. R04 remains excluded. |
| I181 | Native And Panic-Boundary Security Review | Complete | Completion Commit `aea26ad011af04396ab8588c9326d309538f31a2`; review/matrix disposition only, with no protected implementation. R04 child owners remain pending. |

## Completed This Closeout

| ID | Codename | Final State | Completion Evidence |
|---|---|---|---|
| I169 | Transactional Batched Steering Turn | **Complete (2026-08-06)** | PR #131 merged at `685d3b4f4088a172551f8c844a89f5dee9469430`; exact accepted Head `90165cace4625c0f27616b3e1b9871bcb6a10186`; CI run `31010166558`; rebuilt real-terminal acceptance passed; TUI-044 Complete; ADR-056 Accepted; Issue #119 completed. Issue #136 remains independent and non-blocking. |
| I170 | Windows Workspace Validation Unblocker | Complete (2026-08-01) | PR #126 squash-merged at `592254d73a98166df48da0139a02df67e9cd2cd6`; exact implementation Head `8cfe8edb2dbda581244f583fb809591391a54298`; CI run `30705366763`; walkthrough artifact `8820174164`; TOOL-023-A/C Complete; ADR-057 Accepted. |
| I176 | CLI Session Handler Decomposition | **Complete (2026-08-07)** | Completion Commit `1de3243d`; PR #159 merged at `37c557271b906664022476bd2775c5cd77f2b8ea`; exact-head CI `31160309818` passed Unix/Windows workspace, governance, remote owner reconciliation, installer fixture, and rebuilt CLI smoke. |
| I177 | Agent Session Custody Decomposition | **Complete (2026-08-07)** | Completion Commit `f505eea8` (squash merge of implementation `786aa571`); PR #162; exact-head CI `31166594367` passed Unix/Windows workspace, governance, remote owner reconciliation, installer fixture, and rebuilt CLI smoke. |
| I178 | Pending Submission Store Decomposition | **Complete (2026-08-07)** | Completion Commit `f92634803560dc50e0b15ca8d7d511e9928c983f` (squash merge of source implementation `c662a7e6`); PR #165; exact-head CI `31180591881` passed Unix/Windows workspace, governance, remote owner reconciliation, installer fixture, and rebuilt CLI smoke. |
| I179 | Core Tool Facade Decomposition | **Complete (2026-08-07)** | Completion Commit `dafc9be08736aee91e0f9cdd92e5226930808061` (squash merge of source implementation `63d494c5`); PR #168; exact-head CI `31189425069` passed Unix/Windows workspace, governance, remote owner reconciliation, installer fixture, and rebuilt CLI smoke. |
| I180 | Architecture Documentation Truth | **Complete (2026-08-08)** | Completion Commit `10cceec6aeb9089fe9c830355992c8fc60430d63` (squash merge of source implementation `fd8ac75d`); PR #171; exact-head CI `31238721507` passed Unix/Windows workspace, governance, remote owner reconciliation, installer fixture, and rebuilt CLI smoke. R04 remains excluded. |
| I181 | Native And Panic-Boundary Security Review | **Complete (2026-08-08)** | Completion Commit `aea26ad011af04396ab8588c9326d309538f31a2`; PR #174 exact head `12963970` passed CI `31255683335`, governance validation, and merge-time CAS. Review-only disposition; R04 implementation remains Partial. |

I169's accepted residuals remain explicit:

- Issue #136 owns direct `/delete` cleanup-failure recovery-command wording only;
- queue editing/reordering, persistent cross-Session steering, retry of a started terminal Turn,
  broader shutdown and general persistent tasks remain separately owned;
- no release or REL-002 readiness claim is made.

I170's accepted residuals remain explicit:

- timeout cleanup is guaranteed for the direct shell child, not the complete descendant tree;
- TOOL-023-B still owns timeout default/configuration;
- a PowerShell lexer/parser, PowerShell 7 selection and Job Object lifecycle require separate decisions.

## Recent Non-Terminal / Completed Context

| ID | State | Notes |
|---|---|---|
| I168 | Complete (2026-07-30) | Provider terminal outcome integrity; completion commit `86262d02`. |
| I167 | Complete (2026-07-29) | Approval option contrast; implementation `3356aac`. |
| I166 | Complete (2026-07-28) | Interrupt shortcut reliability; automated and maintainer Alacritty acceptance passed. |
| I165 | Complete (2026-07-28) | Growing conversation composer continuity; all human acceptance cases passed. |
| I164 | Paused (2026-07-28) | Startup-inline target superseded; no Completion Commit. |
| I163 | Complete (2026-07-28) | Policy-controlled linked skill discovery. |
| I157 | Complete (2026-07-30 correction) | Provider removal/credential clear stale-snapshot concurrency correction. |
| I156 | Complete (2026-07-27) | Narrow-viewport and resize robustness; maintainer Alacritty walkthrough passed. |

## I169 Completion Evidence

- [x] activated from the recorded current-main architecture baseline;
- [x] recovery PR #120 and its branch remained immutable;
- [x] structured transaction, journal, lifecycle, Scheduler, exact-request and replay behavior implemented;
- [x] independent review findings remediated;
- [x] exact accepted Head `90165cace4625c0f27616b3e1b9871bcb6a10186` passed CI `31010166558`;
- [x] rebuilt release binary completed the real-terminal A/B/C, restart, fork, delete and recovery walkthrough;
- [x] PR #131 merged at `685d3b4f4088a172551f8c844a89f5dee9469430`;
- [x] TUI-044 marked Complete and ADR-056 marked Accepted;
- [x] Issue #119 closed as completed;
- [x] Issue #136 retained as a separately owned non-blocking residual.

## History

The prior full iteration registry and non-terminal inventory remain available at:

- [`ITERATIONS-INDEX-pre-I170-closeout-2026-08-01.md`](ITERATIONS-INDEX-pre-I170-closeout-2026-08-01.md)

Individual plans and completion records remain under `docs/iterations/`; this compact index does not
replace or rewrite them.
