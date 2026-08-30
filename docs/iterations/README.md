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
| I189 | PERM-006-A Structured Permission Decisions | Complete / Closed | Completion Commit `6b577d6a`; PR #356 merged as `54241bdd` after exact-head CI `32511672926`, independent Agent-role permission/security/code review `5376591491` and CAS. PERM-006-B/I219 is separately Complete/Closed through PR #368 merge `de79ad46`; no authority transfers back into I189. |
| I190 | Change-Aware CI Routing | Complete | Completion Commit `a69ffa30afed16271885d4ef3d11931ab3189673` implements trusted-base fail-closed routing; probe Completion Commit `01721f683d0c09ad5f5f9e98360da15cd5155c48` proves the real reduced path. GOV-006 separately owns the unclaimed case-normalization residual. |
| I191 | Non-Interactive Terminal Containment | Complete | Completion Commit `512ff32f389167364c02e7058151879b9ce6859a`; final head `6b2dbdb5`, CI `31587076213`, independent review `5274917099`; I188 now consumes its `setsid` evidence without reopening I191. |
| I192 | Session Runtime Recovery Closure | Complete | Completion Commit `512ff32f389167364c02e7058151879b9ce6859a`; final head `6b2dbdb5`, CI `31587076213`, independent review `5274917099`; its SESSION-008-B residual was later completed by I193. |
| I193 | SESSION-008-B Durable Partial-Turn Finalization | Complete | Completion Commit `404d7a4bf5b9c7dedeae479fe91fa5400b42d411`; PR #216 merged as `1b5461cd`; exact-head CI `31691761892`; disclosed role audits `5287961007`/`5287989820`. RUNTIME-005 retains its owner-defined gates. |
| I194 | Desktop Renderer, Host, And Repository Boundary | Complete | Completion Commit `0a47208ce6fad23c706ebede8b3d07111b9303dc`; PR #215 merged as `1beaca68`; exact-head CI `31687636396`; independent approval `5278769979`. ADR-059 remains Proposed and later renderer gates remain separate. |
| I195 | Dashboard Read-Only Visual Shell | Complete / Closed | Completion Commit `490503db`; PR #233 merged after exact-head CI `32087223234`, independent AI technical review, human browser acceptance and merge-time CAS. Excluded live/write/control/remote work remains separately governed. |
| I196 | Canonical Work Domain Decision And Migration Contract | Complete / Closed | Completion Commit `779a4c71`; PR #291 merged as `1467a561` after exact-head CI `32101943484` and independent architecture review. P0 changed no Work Graph, Evaluator, persistence, API, Desktop, Dashboard or TUI behavior. |
| I197 | Permission Prompt Layout Anchor Stability | Complete / Closed | Issue #125 / TUI-045; implementation merged as `d98f37e7`; natural-person layout validation closed 2026-08-28. |
| I198 | Optional Skill Triggers Compatibility | Complete / Closed | Completion Commits `f719ed91`/`fedd6fac`; I198 delivered omitted/empty/list compatibility and I232/SKILL-005 closed malformed-input activation diagnostics with exact-head CI `33141878176` and independent review `5448628671`. |
| I199 | Thinking Preview Wrap And Bounded Height | Complete / Closed | Completion Commits `938c9edb`/`558b76d3`/`14bf4e60`/`add84074`/`de24bffd`; PR #297 merged as `5fc814b5` after exact-head CI `32138003207`, maintainer terminal acceptance and independent review. TUI-056/#298 remains separate. |
| I200 | No-Op History Scroll State Stability | Complete / Closed | Completion Commit `3afeeb28`; automated gates and the maintainer-approved touchpad substitute passed the native-device matrix. TUI-061/#334 separately owns the continuation-padding regression. |
| I201 | Tool-Call Placeholder Suppression | Complete / Closed | Issue #111 / TUI-043; implementation merged as `7f5a6df2`; approval, denial and cancel validation closed 2026-08-28. |
| I202 | Dashboard Availability In The Logo Prefix | Complete / Closed | Completion Commit `6d3f85ea9f7e76f617ec9716f17ecdd0f9dd0772`; PR #230 merged as `e0cc782a`; exact-head CI `31775126382`, independent security approval `5290402214`, real-terminal acceptance and CAS `5290414997` passed. SEC-002 owns the separate opt-in token-delivery decision. |
| I203 | v0.8.0 GitHub And Crates.io Publication | Complete / Closed | Completion Commits `b0354ae6`/`d8e1aa26`/`d5de4a65`; PR #264 merged as `f425e7bc`. Workflow `31953951828` completed the GitHub Release before all 20 crates published; external CLI install and registry-only runtime fixture passed. |
| I204 | v0.8.0 Release-Candidate Registry Readiness | Complete / Closed | Completion Commit `f46094e3`; PR #260 merged as `7c10afe3`; reviewed conditional GO for preparing I203 claim only. Fresh I203 claim remains required; no release or publication was authorized by I204. |
| I206 | Esc-Cancelled Steering Activation | Complete / Closed | Completion Commit `9d7c87cb`; PR #411 merged from exact head `c742eea5` after CI `33063880465` attempt 2, real-terminal evidence, independent approval `5438243363` and CAS. |
| I207 | Steering Wrap Padding Contract | Planned / Unclaimed | `TUI-049`; continuation lines must honor shared horizontal padding. No implementation branch or authorization. |
| I208 | Steering Boundary Insertion | Planned / Unclaimed | `TUI-050`; insertion timing after a model response or tool-call boundary. No implementation branch or authorization. |
| I205 | PR Workflow Throughput Simplification | Complete / Closed | Completion Commit `2e2cf04b`; PR #287 merged as `0394e264`; exact-head CI `32094384772` and independent technical audit `5323234878` passed. Audit selects atomic claim activation as a separately claimable follow-up; no executable governance change. |
| I209 | Resumed Session Interactivity Under Provider Delay | Closed | TUI-051 / Issue #272 is Complete at Completion Commit `2eff6285`, with source implementation `7b82fea6`/`7d90def8`, exact-head CI `32025371877`, real-terminal acceptance and independent agent audit `5316533941`. Retry-progress acceptance remains transferred to I210. |
| I210 | Provider Retry Progress Contract | Complete / Closed | PROVIDER-006 / Issue #278; implementation merged as `9d5c8a71`; live retry-status acceptance closed 2026-08-28. |
| I211 | Deferred Human Review And Acceptance Batch | Complete / Closed | Completion Commits `b7d55a0d`/`7c333d98`; PR #331 merged as `97dbf35f` after CI `32372514265`, independent review and CAS. Four failed source children retain separate corrective owners. |
| I212 | Catalog-Assisted Custom-Model Context Window | Complete / Closed | Completion Commit `5a1709cb`; PR #318 CI/review/CAS and integrated exact/prefix/override/unknown no-request walkthrough passed. |
| I213 | Dashboard Live Activity And Log Viewer | Complete / Closed | Completion Commit `9f963d0ca662f334fe007d0fcfc857640a2a5bd6`; PR #372 merged after CI `32680547034` and review `5390029881`. |
| I214 | Bounded Shutdown Contract Decision | Complete / Closed | Completion Commit `6719c876`; PR #338 merged as `fc70e396` after exact-head CI `32449605985`, independent architecture review `5365529351` and CAS. ADR-063 is Accepted; B/I216 and C/I217 later completed separately. |
| I215 | Local Convergence And Stage Validation | Complete / Closed | Completion Commit `06e61e3c`; PR #341 merged as `81a603b4` after CI `32442052401`, review `5365129718` and CAS. No product/runtime authority. |
| I216 | Bounded Shutdown Coordinator And Admission Fence | Complete / Closed | Completion Commit `c123328d`; PR #345 merged as `020de694` after exact-head CI `32459530911`, independent runtime review `5367434951` and CAS. C/I217 later completed at `44e840d7`. |
| I217 | Ordered Finalizer Registry And Durable Closure | Complete / Closed | Completion Commit `44e840d7`; PR #348 merged as `6e5fa8c3`; exact-head CI `32475052535`; independent runtime architecture review `5369328072`. |
| I218 | Auto Permission Security Decision | Complete / Closed | Completion Commit `a289a07f`; exact-head CI `32505438495`, independent Agent-role security review `5372825090`, CAS and PR #353 merge `c129d4a5` passed. ADR-064 Accepted; no behavior or later child authority. |
| I219 | PERM-006-B First-Class Scoped Grants | Complete / Closed | Completion Commits `56436027`/`d0c96048`; exact head `97028ac0` passed CI `32579790496`, independent Agent-role review `5381051760`, CAS and PR #368 merge `de79ad46`. C and TOOL-024 stay blocked. |
| I220 | PERM-006-C Agent-Owned Pipeline Decision | Complete / Closed | Completion Commits `c21bb7f3`/`820586ea`; PR #373 merged as `5d2d2dcf`; ADR-067 Accepted. I221 separately owns implementation. |
| I221 | PERM-006-C Agent-Owned Pipeline Implementation | Complete / Closed | Completion Commit `49d1546c`; PR #376 merged as `f9e6706d` after exact-head CI `32640691772`, independent permission/security/API approval `5386153429` and CAS. |
| I222 | TOOL-024-B Managed Background Execution Core | Complete / Closed | Completion Commit `8671edf45c168612bfa4a4bbb65a9847026e1b96`; PR #382 merged after exact-head CI `32690533253` and protected-scope reviews. TOOL-024-C/D and I223 remain separate. |
| I223 | Issue #59 Deferred Human Validation Cleanup | Complete / Closed | Completion Commit `a5fbc22e`; evidence merge `00bf2d5d`, Windows device run `32958236636`, and owner-first validation closeout completed. No product behavior authority. |
| I224 | TOOL-024-C Model-Readable Process Job Control | Complete / Closed | Completion Commit `60b0367cf749397bf1167e189e820e82e32baf03`; PR #386 merged after exact-head CI `32719779528`, independent approval `5394777902` and merge-time CAS. D1/D2 and I223 remain separate. |
| I225 | TOOL-024-D1-A Windows Job Object Decision | Complete / Closed | ADR-068 accepted by PR #391 merge `0021690e`; Completion Commit `fca45c46`; D1-B remains separately governed. |
| I226 | TOOL-024-D1-B Windows Job Object Process-Tree Ownership | Complete / Closed | Completion Commit `d4d7cb25`; PR #394 merged from exact candidate `83557863`, CI `32849330531`, independent review `5410840103`. |
| I228 | TOOL-024-D2 Interactive Projection And Cross-Platform Acceptance | Complete / Closed | Completion Commit `a5fbc22e`; PR #403 exact head `e65f9b49` passed CI `32937579899`, independent review `5421558305` and merge-time CAS. I223 remains separate. |
| I229 | Permission-Mediated Tool Activity Correlation | Complete / Closed | Completion Commit `5a9b6589`; PR #414 merged from exact head `b2179d33` after CI `33074348512` attempt 2 and independent approval `5439760729`. |
| I230 | Permission Prompt Composer-Relative Docking | Complete / Closed | Completion Commit `b29a3d92`; PR #418 merged from exact head `1cb595ce` after CI `33130918248` attempt 2 and bounded single-maintainer CAS record `5447132762`. |
| I231 | Initial Connection And Queue Status Sequencing | Complete / Closed | Completion Commit `e4cbb714`; PR #421 exact head `38c4f030` passed CI `33136176771` and bounded single-maintainer CAS `5447796064`. |
| I232 | Invalid Skill Diagnostic Visibility | Complete / Closed | Completion Commit `fedd6fac`; implementation source `fb47b0c2`, exact-head CI `33141878176`, independent Skill/CLI review `5448628671`. Fail-closed invalid-Skill discovery and bounded explicit activation diagnostics are complete. |
| I233 | Auto Configuration And Session Command | Complete / Closed | PERM-007-B / Issue #188; implementation merged as `c536e190`. Config/session `/auto` only; no model or permission-result authority. |
| I234 | Bounded Model-Assisted Permission Resolver | Complete / Closed | PERM-007-C / Issue #188; Completion Commit `7ddba098b5929e593fff94b9d3f5fd10f2fb35c1`, PR #434 merged as `c5be0109b3da4f81e221fa37f734af2431e35255`. |
| I236 | PERM-007-D Cross-Surface Conformance | Complete / Closed | Completion Commit `5cb6ddc5b6e025ca9f116401f85aeb9a90cc8bba`; PR #438 merged after exact-head CI `33291156177` and independent approval `5466806184`. |
| I237 | Canonical Work Domain And Todo Compatibility | Review / Claimed | Claim PR #440 merged as `3f6f036d`; stable local implementation candidate is ready for one implementation PR and independent review. P2-P4 remain blocked. |
| I235 | PERM-007-C Atomic Create Capability Decision | Complete / Closed | Completion Commit `71acbe0c`; PR #432 accepted the decision-only directory-capability contract. |
| I227 | Tombstone-Pruning Fixture Performance | Complete / Closed | Completion Commit `7b64a08b`; PR #399 merged as `d02915e0` after exact-head CI `32839820741` (5/5) and independent Agent-role approval `5409698923`. |

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
