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

I158 and I171 completion evidence: `Completion Commit: 56f419f7` (source implementation/audit
closeout evidence; status synchronization commits do not self-certify completion).

| ID | Codename | State | Activation / Completion Gate |
|---|---|---|---|
| I158 | Tool Registration Composition Consolidation | Complete | Completion Commit `c88c1d1a`; scheduler/status exceptions and documentation closeout accepted. TUI-037 remains independent. |
| I171 | Workspace Architecture Rebaseline | Complete | Completion Commit `56f419f7`; source v0.7.0 audit/register evidence `c88c1d1a`; bounded remediation owners validated; no production refactor. |
| I159 | `talos-tools` Lightweight Feature Boundary | Complete / Closed | Completion Commit `d886917e45d5ca0f110e111b966cd379485e3580` plus cfg follow-up `34c09b14`; exact head `33a2c6ff` passed CI `31801484313`, approval `5293622712` and CAS, then PR #236 merged as `f79c1ead`. |
| I160 | Shared CLI And Runtime Internal Composition | Complete / Closed | Completion Commit `0524e82f`; PR #240 merged as `97556149`, closeout PR #241 merged as `2d48bd2c`; I161 and I162 are Complete/Closed. |
| I161 | Sandbox Fallback And Coding Preset | Complete / Closed | Completion Commits `74c5502d`/`3ca2ec62`; PRs #250/#251 merged after exact-head independent approvals and CI. I162 is Complete/Closed through Completion Commit `077b347d` and PR #255 merge `16564ba0`. |
| I162 | v0.6 SDK Fixture And Publication Readiness | Complete / Closed | Completion Commit `077b347d`; PR #255 merged as `16564ba0` after exact-head CI `31891263313` and independent approval `5302842269`; reviewed readiness NO-GO keeps I203 blocked. |
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
| I182 | Symbol Traversal Containment | Complete | Completion Commit `ae31242bdac4807599146bfb4847bcac52712bbf`; exact source head `4b968823`, CI `31266112256`, independent review `5230395611`, and merge-time CAS passed. R04 remains Partial. |
| I183 | Bundled SQLite ADR Reconciliation | Complete | Completion Commit `edf903aa96574043294923ad60b0cefe9730f8c4`; final source `0d9a5a7b`, CI `31349520295`, independent review `5235367999` and merge-time CAS passed. R04 remains Partial; AG-11 remains separate and AG-12 is active under I185. |
 | I184 | TUI-046-A Native Selection Policy | Complete | Completion Commit `f98488277803ee26180100089a48ef850939234b`; reviewed head `24e15db8d9df852c07fe08cc79ccc670fda36d27`; review `5237824299`; CI `31370219799`. B is separately claimable; cross-platform testing is deferred to B acceptance. |
 | I185 | SQLite Validator Policy Integrity | Complete | Completion Commit `af9783229bfc8ee592813440ecfcdb6efc90a3c2`; exact head `45f70802`, CI `31556720252`, independent review `5261491057`. |
 | I186 | TUI Visible-Cell Selection And Copy | Complete | Completion Commit `a5115f5ce6484512ceb83867f72fa9b47ab8f5fc`; final head `313e47e5`, CI `31481069023`, independent review `4905391760`; terminal matrix bound to `70b51e28`. |
 | I187 | SESSION-008-A Partial-Turn Lifecycle Decision | Complete | Completion Commit `e288afb5d97026f7ccb3ce0f519a4a81f99fe104`; final head `46549e82`, CI `31553007431`, proposal review `5261130488`; ADR-058 acceptance is bound to the reviewed closeout. |
| I188 | TOOL-024-A Background Job Lifecycle Contract | Closed / Delivered | Owner Completion Commit `245eddebae762d1d0c7ee796baea50d0bb080bd5`; PR #228 merged as `1db1211e` from exact head `d7d4fe7a` after CI `31995198205`, independent security review `5312482823` and merge-time CAS. Production B remains dependency-gated and Windows remains fail-closed pending D. |
| I189 | PERM-006-A Structured Permission Decisions | Planned / Claimed | Claim merge `0df88638` establishes only the behavior-preserving A foundation for Issues #52/#53; no implementation has started and protected-scope review remains mandatory. |
| I190 | Change-Aware CI Routing | Complete | Completion Commit `a69ffa30afed16271885d4ef3d11931ab3189673` implements trusted-base fail-closed routing; probe Completion Commit `01721f683d0c09ad5f5f9e98360da15cd5155c48` proves the real reduced path. GOV-006 separately owns the unclaimed case-normalization residual. |
| I191 | Non-Interactive Terminal Containment | Complete | Completion Commit `512ff32f389167364c02e7058151879b9ce6859a`; final head `6b2dbdb5`, CI `31587076213`, independent review `5274917099`; I188 now consumes its `setsid` evidence without reopening I191. |
| I192 | Session Runtime Recovery Closure | Complete | Completion Commit `512ff32f389167364c02e7058151879b9ce6859a`; final head `6b2dbdb5`, CI `31587076213`, independent review `5274917099`; its SESSION-008-B residual was later completed by I193. |
| I193 | SESSION-008-B Durable Partial-Turn Finalization | Complete | Completion Commit `404d7a4bf5b9c7dedeae479fe91fa5400b42d411`; PR #216 merged as `1b5461cd`; exact-head CI `31691761892`; disclosed role audits `5287961007`/`5287989820`. RUNTIME-005 retains its owner-defined gates. |
| I194 | Desktop Renderer, Host, And Repository Boundary | Complete | Completion Commit `0a47208ce6fad23c706ebede8b3d07111b9303dc`; PR #215 merged as `1beaca68`; exact-head CI `31687636396`; independent approval `5278769979`. ADR-059 remains Proposed and later renderer gates remain separate. |
| I195 | Dashboard Read-Only Visual Shell | Complete / Closed | Completion Commit `490503db`; PR #233 merged after exact-head CI `32087223234`, independent AI technical review, human browser acceptance and merge-time CAS. Excluded live/write/control/remote work remains separately governed. |
| I196 | Canonical Work Domain Decision And Migration Contract | Complete / Closed | Completion Commit `779a4c71`; PR #291 merged as `1467a561` after exact-head CI `32101943484` and independent architecture review. P0 changed no Work Graph, Evaluator, persistence, API, Desktop, Dashboard or TUI behavior. |
| I197 | Permission Prompt Layout Anchor Stability | Review / Claimed - human validation deferred | Issue #125 / TUI-045; final head `9fce4f13` merged through PR #305 as `d98f37e7` after exact-head CI, Agent technical review and CAS. Human/manual rows remain unpassed in Issue #302 / I211. |
| I198 | Optional Skill Triggers Compatibility | Planned / Unclaimed | Issue #155 / SKILL-004; keep unactivated until its separate claim and compatibility decision checkpoint. |
| I199 | Thinking Preview Wrap And Bounded Height | Complete / Closed | Completion Commits `938c9edb`/`558b76d3`/`14bf4e60`/`add84074`/`de24bffd`; PR #297 merged as `5fc814b5` after exact-head CI `32138003207`, maintainer terminal acceptance and independent review. TUI-056/#298 remains separate. |
| I200 | No-Op History Scroll State Stability | Review / Claimed - human validation deferred | Issue #79 / TUI-042; implementation `3afeeb28` merged through PR #301 as `9628e183` after exact-head CI, Agent technical review and CAS. Natural-person and mouse/touchpad rows remain unpassed in VALIDATION-002/I211/#302. |
| I201 | Tool-Call Placeholder Suppression | Active / Claim Pending (#306) | Issue #111 / TUI-043; governance-only claim proposed from `main@8069ea6a`; no implementation branch or behavior authority exists before merge. |
| I202 | Dashboard Availability In The Logo Prefix | Complete / Closed | Completion Commit `6d3f85ea9f7e76f617ec9716f17ecdd0f9dd0772`; PR #230 merged as `e0cc782a`; exact-head CI `31775126382`, independent security approval `5290402214`, real-terminal acceptance and CAS `5290414997` passed. SEC-002 owns the separate opt-in token-delivery decision. |
| I203 | v0.8.0 GitHub And Crates.io Publication | Complete / Closed | Completion Commits `b0354ae6`/`d8e1aa26`/`d5de4a65`; PR #264 merged as `f425e7bc`. Workflow `31953951828` completed the GitHub Release before all 20 crates published; external CLI install and registry-only runtime fixture passed. |
| I204 | v0.8.0 Release-Candidate Registry Readiness | Complete / Closed | Completion Commit `f46094e3`; PR #260 merged as `7c10afe3`; reviewed conditional GO for preparing I203 claim only. Fresh I203 claim remains required; no release or publication was authorized by I204. |
| I206 | Esc-Cancelled Steering Activation | Planned / Unclaimed | `TUI-048`; accepted steering must become one runnable Session turn after active-turn Esc cancellation. No implementation branch or authorization. |
| I207 | Steering Wrap Padding Contract | Planned / Unclaimed | `TUI-049`; continuation lines must honor shared horizontal padding. No implementation branch or authorization. |
| I208 | Steering Boundary Insertion | Planned / Unclaimed | `TUI-050`; insertion timing after a model response or tool-call boundary. No implementation branch or authorization. |
| I205 | PR Workflow Throughput Simplification | Complete / Closed | Completion Commit `2e2cf04b`; PR #287 merged as `0394e264`; exact-head CI `32094384772` and independent technical audit `5323234878` passed. Audit selects atomic claim activation as a separately claimable follow-up; no executable governance change. |
| I209 | Resumed Session Interactivity Under Provider Delay | Closed | TUI-051 / Issue #272 is Complete at Completion Commit `2eff6285`, with source implementation `7b82fea6`/`7d90def8`, exact-head CI `32025371877`, real-terminal acceptance and independent agent audit `5316533941`. Retry-progress acceptance remains transferred to I210. |
| I210 | Provider Retry Progress Contract | Planned / Unclaimed | PROVIDER-006 / Issue #278; accept an ADR and establish an effective claim before any public contract or implementation change. |
| I211 | Deferred Human Review And Acceptance Batch | Planned / Unclaimed | VALIDATION-002 / Issue #302; evidence-only cleanup after I200/I197/I201/I198 implementation dispositions. No product implementation or gate waiver. |

## Completed This Closeout

| ID | Codename | Final State | Completion Evidence |
|---|---|---|---|
| I169 | Transactional Batched Steering Turn | **Complete (2026-08-06)** | PR #131 merged at `685d3b4f4088a172551f8c844a89f5dee9469430`; exact accepted Head `90165cace4625c0f27616b3e1b9871bcb6a10186`; CI run `31010166558`; rebuilt real-terminal acceptance passed; TUI-044 Complete; ADR-056 Accepted; Issue #119 completed. Issue #136 remains independent and non-blocking. |
| I202 | Dashboard Availability In The Logo Prefix | **Complete (2026-08-14)** | Completion Commit `6d3f85ea9f7e76f617ec9716f17ecdd0f9dd0772`; PR #230 merged as `e0cc782a475c2e5baceb31f2a125f1e268af7ecf`; CI `31775126382`, approval `5290402214`, real-terminal acceptance and CAS `5290414997` passed. SEC-002 owns the unclaimed token-delivery residual. |
| I170 | Windows Workspace Validation Unblocker | Complete (2026-08-01) | PR #126 squash-merged at `592254d73a98166df48da0139a02df67e9cd2cd6`; exact implementation Head `8cfe8edb2dbda581244f583fb809591391a54298`; CI run `30705366763`; walkthrough artifact `8820174164`; TOOL-023-A/C Complete; ADR-057 Accepted. |
| I176 | CLI Session Handler Decomposition | **Complete (2026-08-07)** | Completion Commit `1de3243d`; PR #159 merged at `37c557271b906664022476bd2775c5cd77f2b8ea`; exact-head CI `31160309818` passed Unix/Windows workspace, governance, remote owner reconciliation, installer fixture, and rebuilt CLI smoke. |
| I177 | Agent Session Custody Decomposition | **Complete (2026-08-07)** | Completion Commit `f505eea8` (squash merge of implementation `786aa571`); PR #162; exact-head CI `31166594367` passed Unix/Windows workspace, governance, remote owner reconciliation, installer fixture, and rebuilt CLI smoke. |
| I178 | Pending Submission Store Decomposition | **Complete (2026-08-07)** | Completion Commit `f92634803560dc50e0b15ca8d7d511e9928c983f` (squash merge of source implementation `c662a7e6`); PR #165; exact-head CI `31180591881` passed Unix/Windows workspace, governance, remote owner reconciliation, installer fixture, and rebuilt CLI smoke. |
| I179 | Core Tool Facade Decomposition | **Complete (2026-08-07)** | Completion Commit `dafc9be08736aee91e0f9cdd92e5226930808061` (squash merge of source implementation `63d494c5`); PR #168; exact-head CI `31189425069` passed Unix/Windows workspace, governance, remote owner reconciliation, installer fixture, and rebuilt CLI smoke. |
| I180 | Architecture Documentation Truth | **Complete (2026-08-08)** | Completion Commit `10cceec6aeb9089fe9c830355992c8fc60430d63` (squash merge of source implementation `fd8ac75d`); PR #171; exact-head CI `31238721507` passed Unix/Windows workspace, governance, remote owner reconciliation, installer fixture, and rebuilt CLI smoke. R04 remains excluded. |
| I181 | Native And Panic-Boundary Security Review | **Complete (2026-08-08)** | Completion Commit `aea26ad011af04396ab8588c9326d309538f31a2`; PR #174 exact head `12963970` passed CI `31255683335`, governance validation, and merge-time CAS. Review-only disposition; R04 implementation remains Partial. |
| I182 | Symbol Traversal Containment | **Complete (2026-08-09)** | Completion Commit `ae31242bdac4807599146bfb4847bcac52712bbf`; PR #177 exact source head `4b96882307173ded8264aa1c45cce129707ff65f` passed CI `31266112256`, independent review `5230395611`, and merge-time CAS. R04 remains Partial. |
| I190 | Change-Aware CI Routing | Complete (2026-08-12) | Completion Commit `a69ffa30afed16271885d4ef3d11931ab3189673`; reduced-probe Completion Commit `01721f683d0c09ad5f5f9e98360da15cd5155c48`; exact implementation CI `31560789644`, independent review `5262374485`, and reduced run `31564461023` passed. |

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
