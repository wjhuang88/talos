# DESKTOP-001-MISSION-UX: Mission-First Desktop Workflow

**Status**: Refinement / Unclaimed
**Type**: Product Story
**Story ID**: DESKTOP-001-MISSION-UX
**Parent Epic**: DESKTOP-001
**Source**: Existing Desktop product/design baseline and Issue #29
**Selected Iteration**: None; candidate successor after I285 receives an explicit disposition

## Identity / Goal / Value

Desktop users need the product to guide work through an explicit Mission lifecycle instead of
presenting a live Runtime conversation as the primary task model. The current host can execute real
work, but it does not yet deliver the designed Mission-first workflow or visual experience.

## Scope

- Shape a requested outcome into a reviewable Mission/Goal/WorkUnit plan before execution.
- Let the user inspect and revise the plan, establish its baseline, and explicitly start execution.
- Make the live task surface Mission-first: canonical goal/work state and progress are primary;
  transcript and raw tool output remain available as supporting detail.
- Align Overview, Plan, Changes, Verification, and Delivery with actual shared work, artifact,
  evaluation, and delivery state where those contracts provide it.
- Present task identity using a user-meaningful Mission title while preserving stable underlying
  identity and explicit workspace/session boundaries.
- Recreate the four archived design surfaces at high fidelity, including information hierarchy,
  page composition, spacing, controls, and representative states; matching only the color palette
  or using fixture-only lookalikes does not pass.

## Exclusions

- No Desktop-owned durable Mission, Goal, Evaluation, approval, or permission authority.
- No second work engine, duplicate mutable Todo/Work Graph, or inferred/fabricated PASS or Delivery.
- No unattended Mission orchestration, multi-client behavior, new permission policy, or release.
- Do not fold this product feature into I285, whose published scope excludes new features.

## Dependencies

- I285 receives a truthful terminal disposition; its H1-H6 evidence and remaining acceptance stay
  owned by I285 and existing Issue #29.
- Reconcile the Desktop host with shared `talos-core::work` and WORK-001 projection, evaluation,
  and Mission-gate contracts before implementation selection.
- Verify whether existing Runtime/session APIs support proposal, baseline approval, revisioning,
  and Mission-first recovery. Missing shared contracts must be separately owned; this Story cannot
  create a Desktop-local substitute.
- Real preset configuration/management remains under DESKTOP-002 / Issue #308 and its dependencies;
  this Story must not present fixture preset choices as if they configure live execution.

## Decision Links And Constraints

- [`DESIGN.md`](../../design/talos-desktop/DESIGN.md): global New Task flow and task-local navigation;
  mock fixtures are not evidence of working live behavior.
- [`REFERENCES.md`](../../design/talos-desktop/REFERENCES.md): four archived images and source hashes;
  visual acceptance compares against these references, subject to the accepted Settings amendment.
- [`talos-desktop-goal-oriented-workspace.md`](../../proposals/talos-desktop-goal-oriented-workspace.md):
  Mission is primary, shaping precedes execution, and structured state is not transcript convention.
- [`WORK-001`](WORK-001-goal-oriented-work-evaluation-foundation.md) and [P4](WORK-001-E-mission-gate-ui-neutral-projection.md):
  consume shared work/evaluation authority; do not add a parallel Desktop model.
- [`ADR-059`](../../decisions/059-desktop-renderer-host-motion-boundary.md): GPUI is an isolated
  presentation host; business lifecycle remains Runtime-owned.
- [`ADR-061`](../../decisions/061-canonical-work-domain-and-todo-migration.md): canonical work
  domain and Todo compatibility remain authoritative.
- [`ADR-083`](../../decisions/083-shared-evaluation-context-boundary.md): evaluation context and
  Delivery remain shared Runtime-owned; no new durable Mission/Evaluation schema or inferred verdict.

## Acceptance For Behavior

- Given a new outcome, when the user starts a Mission, then Talos presents a proposed structured
  plan without executing work before the user reviews and baselines it.
- Given a proposed plan, when the user edits goals or acceptance, then the baseline/revision visibly
  reflects those edits and execution starts only against the confirmed revision.
- Given a running Mission, when the user opens its primary view, then the current Goal/WorkUnit,
  Mission position, recent semantic activity, and concise changes are primary; transcript/raw tool
  detail is available without replacing the Mission state model.
- Given missing, stale, failed or unavailable evaluation evidence, when the user inspects
  Verification or Delivery, then that state is explicit and success is never inferred.
- Given an existing Mission after restart, when the user resumes it, then stable identity, confirmed
  plan and persisted shared work state are restored without replaying completed tools.

## Acceptance For Visual Fidelity

- [ ] Capture and compare New Task, Task Overview, Preset List, and Preset Detail/Settings at the
  reference viewport and a narrow viewport; the accepted Settings-navigation amendment supersedes
  only the old top-level Presets navigation arrangement.
- [ ] Match the reference hierarchy, primary/secondary regions, alignment, spacing, control
  placement, and key visual states; document and justify any deliberate deviation before claiming
  the surface accepted.
- [ ] Exercise supported surfaces with real product state; fixture rendering alone is not
  functional acceptance. Preset behavior remains governed by #308 and must not be represented as
  live until that dependency is delivered.
- [ ] Verify Chinese and English layout, wrapping, and core interactions against the visual baseline.

## Reference-Surface Acceptance Matrix

All four archived PNG references use the same `1448x1086` baseline viewport. A surface is not
accepted from a screenshot alone: its primary actions must drive the corresponding real state.

| Surface | Required visual structure | Required live behavior | Current status |
|---|---|---|---|
| New Task | Global navigation, outcome entry, preset selector, explicit workspace selector, primary start action, calm single-column hierarchy | Creates or loads the shared Mission proposal; does not execute before plan review/baseline | Current live path submits directly; not conformant |
| Task Overview | Task-local Overview selected; Mission/Goal/WorkUnit hierarchy, current position, recent activity, concise changes and secondary drill-down | Reads shared work/evidence state; no fixture-only progress or inferred completion | Current live page is prompt/output-oriented; not conformant |
| Preset List / Settings | Settings-owned preset management, default marker, list/detail hierarchy, aligned actions and reference spacing | Reads/writes the authorized preset source; fixture templates cannot claim live configuration | Current fixture/template path is not live preset authority |
| Preset Detail / Edit | Detail title, basic information, instructions, model roles and capability controls with reference hierarchy | Persists through the #308 contract when that dependency is implemented | Blocked by DESKTOP-002 / #308 |

### Visual Comparison Protocol

For each surface, capture the exact reference viewport and a narrow laptop viewport; compare the
same state with an image diff or annotated review. Record deliberate differences by reason (accepted
Settings navigation amendment, platform-native control behavior, or responsive adaptation). A
palette match, a fixture screenshot, or a page that merely contains the same labels is insufficient.

## Uncertainty And Validation Path

**Confirmed:** current `render_live` is prompt/output-oriented; the Mission progress and task-local
tab prototype is fixture-backed. The live Desktop host reads a per-session Todo-backed WorkGraph
projection through `SessionManager::load_work_graph_read_only`; the repository projects Todo items
and dependency edges as WorkUnits. Shared types include Mission/Goal identities, evaluation
subjects and a fail-closed Mission Delivery gate, but those facts do not themselves implement a
user-facing shaping or confirmed-plan lifecycle. The configured host reaches real
Runtime/provider/tool/permission paths. This establishes real execution plumbing, not design
conformance.

**Unknown:** whether other existing Runtime/session APIs expose the complete proposal, baseline,
revision, persistence and recovery lifecycle. Before marking Ready, map each user action to shared
contracts and register any missing contracts as separate dependencies; do not infer that a
storage-neutral WorkGraph or Delivery gate is a persisted Mission workflow.

## Readiness Contract Map — 2026-09-29

| Product capability | Existing reusable boundary | Current gap before MISSION-UX can be Ready |
|---|---|---|
| Mission/Goal/WorkUnit identity and revision | `talos_core::work::{WorkIdentity, WorkNode, WorkGraph}` | No Desktop-facing proposal/baseline transition or mutation contract |
| Todo/work projection | `SessionManager::load_work_graph_read_only` and session Todo repository | Read-only per-session projection; no confirmed Mission plan persistence or revisioned plan editor |
| Goal evaluation and Delivery gate | `RuntimeEvaluationService`, `CompletionClaim`, `MissionGate`, ADR-083 | Explicit evaluation exists, but no live producer binds the user's shaped plan and authoritative criteria |
| Runtime execution | `RuntimeHandle::submit`, interrupt/shutdown, durable Session | Current Desktop submits a prompt directly; no pre-execution baseline admission |
| Session/restart | `SessionManager` durable transcript and workspace task bindings | Need proof that confirmed plan identity/state is restored independently of transcript replay |
| Presets | Current Desktop fixture/template UI; DESKTOP-002 / #308 | Fixture preset selection must not be represented as live environment configuration |
| Visual surfaces | Archived four reference images, DESIGN.md, current GPUI renderer | Live execution page is not the reference Mission Overview/New Task/Settings composition; needs high-fidelity implementation and real-state validation |

MISSION-UX is not Ready until every row has either an existing shared callable contract or a
separately owned prerequisite with an accepted boundary, migration/rollback expectations, and
observable validation. A new Desktop-local store or inferred evaluation result is not an allowed
shortcut.

## State / Status Owners

- Scope/readiness/status: this owner and compact backlog.
- Current I285 limits and discovery: [I285](../../iterations/I285-desktop-integrated-acceptance.md),
  four-week task, and existing Issue #29.
- Parent product direction: [DESKTOP-001](DESKTOP-001-desktop-product-direction.md).
- Operating summary: `docs/BOARD.md` and `docs/iterations/README.md`.

## User-Facing Documentation

When delivered, update Desktop README and bilingual guidance for creating, reviewing, baselining,
starting, resuming, and inspecting a Mission. Until then, do not describe the prompt-first host as
the completed Mission workflow.

## Required Reads

- `docs/design/talos-desktop/DESIGN.md`
- `docs/design/talos-desktop/REFERENCES.md`
- `docs/proposals/talos-desktop-goal-oriented-workspace.md`
- `docs/backlog/active/WORK-001-goal-oriented-work-evaluation-foundation.md`
- `docs/backlog/active/WORK-001-E-mission-gate-ui-neutral-projection.md`
- `docs/backlog/active/DESKTOP-001-D7-desktop-integrated-acceptance.md`
- `docs/iterations/I285-desktop-integrated-acceptance.md`
- `docs/decisions/059-desktop-renderer-host-motion-boundary.md`
- `docs/decisions/061-canonical-work-domain-and-todo-migration.md`
- `docs/decisions/083-shared-evaluation-context-boundary.md`
- `docs/backlog/active/DESKTOP-002-preset-session-environment-templates.md`
