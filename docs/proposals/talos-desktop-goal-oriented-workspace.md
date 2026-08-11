# Talos Desktop Goal-Oriented Workspace Design Baseline

> Status: Design baseline / proposal refinement — **no implementation authorization**
>
> Date: 2026-08-11
>
> Owner direction: `DESKTOP-001`
>
> Source discussion: maintainer product/architecture discussion consolidating the Desktop renderer,
> goal-oriented workflow, Todo evolution, independent evaluation, artifact review, and delivery
> model.

## 1. Purpose

Talos Desktop must not be a graphical clone of the current TUI. Reproducing the same transcript,
slash-command, and tool-log experience inside a native window would add packaging and UI cost
without creating a distinct product capability.

The Desktop product should instead provide a **goal-oriented workspace for delegating, shaping,
supervising, evaluating, and reviewing complex AI work**.

The product-level split is:

- **Talos TUI:** conversation-first, immediate, dense, keyboard-oriented interaction for expert and
  rapid workflows.
- **Talos Desktop:** goal-first, state-centric, visual interaction for longer-running and more
  structured work.

A concise product statement is:

> Define the outcome. Shape the plan. Let Talos execute. Independently evaluate the result. Review
> the delivery.

This document records the design baseline agreed for that direction and defines the scope of a
**future, separate Desktop prerequisite implementation PR**. This document PR itself must not add
Desktop code, create the prerequisite implementation PR, or imply that implementation has been
selected or authorized.

## 2. Product Model: Mission Instead Of Transcript

The primary Desktop object is a **Mission**, not an endlessly growing conversation.

A user begins with a rough outcome such as:

> Add complete Desktop provider configuration support.

Talos does not immediately execute. It first enters a shaping phase and proposes a structured work
model. The user can inspect, reorder, refine, add, remove, or rewrite goals and acceptance criteria
before execution.

Natural language remains available throughout the product, but it is an **editing mechanism for
state**, not the primary state container. For example, when a user says:

> Do not add provider search in the first version; list plus configuration form is enough.

Talos should update the applicable goal and acceptance criteria. The durable product fact is the
changed goal baseline, not a pair of transcript messages saying that the requirement changed.

## 3. Desktop Technology Direction

### 3.1 Selected renderer direction

The Desktop renderer direction is **GPUI**, with a Rust-native product host. Tauri/WebView is not the
current selected route.

The rationale is product fit rather than an abstract preference for native UI. Talos Desktop is
expected to become a text- and workspace-heavy application containing:

- Markdown and code blocks;
- syntax-highlighted source;
- diff and change review;
- file/workspace navigation;
- large lists and panels;
- command palette and keyboard interaction;
- terminal/log/diagnostic views;
- model, provider, permission, Skill, and runtime state;
- IME and high-quality text input;
- long-running Agent activity views.

GPUI is the primary direction for that renderer class. Makepad remains a possible future visual or
shader experimentation path, not the initial Desktop product framework.

### 3.2 Rust-first boundary

The Desktop direction must preserve repository hard constraints:

- Agent/runtime/security/business logic remains Rust-owned;
- no Node.js or Python runtime is introduced as an application dependency;
- native/unsafe implications of GUI dependencies require the repository's normal ADR and security
  review before implementation;
- Desktop renderer code must not become a dependency of `talos-core`, `talos-runtime`, or reusable
  capability crates.

### 3.3 Renderer architecture

The intended direction is multiple independent renderers over shared product/runtime semantics:

```text
                    talos-runtime
                         |
              shared product projection
                         |
                shared text semantics
                  /              \
                 /                \
          talos-tui          talos-desktop
       Ratatui/Crossterm          GPUI
```

The names and exact crate boundaries of future shared presentation/text layers are not authorized by
this proposal. Existing `talos-conversation` already owns a UI-independent product projection and
must be reconciled before creating any new presentation crate. A second real renderer is evidence
for extraction only where the current dependency direction cannot serve both consumers cleanly.

Hard dependency direction:

```text
talos-core / talos-runtime
    !-> GPUI
    !-> talos-desktop

talos-tui
    !-> talos-desktop

talos-desktop
    !-> talos-tui
```

## 4. Mission Lifecycle

The Desktop workflow is modeled as explicit state rather than transcript convention:

```text
INTAKE
  |
  v
SHAPING
  |
  v
BASELINED
  |
  v
EXECUTING
  |
  | executor submits completion claims
  v
EVALUATION_PENDING
  |
  v
EVALUATING
  |\
  | \-- FAIL ----------> REWORK ----> EXECUTING
  |\
  | \-- INCONCLUSIVE --> NEEDS_INPUT
  |
  \---- PASS ----------> COMPLETED
                           |
                           v
                  MISSION_EVALUATING
                           |
                           v
                       DELIVERING
                           |
                           v
                       DELIVERED
```

`CHAT` is intentionally not a lifecycle state. Natural language can be used within each lifecycle
stage, but Mission, Goal, evaluation, artifact, and delivery facts remain structured state.

## 5. Work Graph: One Canonical Planning And Execution Model

### 5.1 Why a graph

The UI may default to a tree projection because hierarchy is easy to understand, but the canonical
model should be a DAG-capable **Work Graph**. Real work contains both containment and dependency:

```text
Mission
├── Goal A
│   ├── Work Unit A1
│   └── Work Unit A2
├── Goal B
└── Goal C

Goal C depends_on Goal A
Goal C depends_on Goal B
```

Containment and execution dependency are different facts and must not be represented by one edge
kind.

### 5.2 Mission

A Mission is the durable unit of delegated outcome. It can span multiple runtime/session instances,
including executor sessions, evaluator sessions, rework sessions, and later reconnects.

A Mission therefore must not be modeled as a child object whose lifetime is owned by one transcript
session.

### 5.3 Work nodes

The first useful node kinds are deliberately small:

```rust
pub enum WorkNodeKind {
    Goal,
    WorkUnit,
}
```

A **Goal** answers:

> What must become true?

A **WorkUnit** answers:

> What should the executor do next?

Goals are user-visible contract/state objects. Work Units are fine-grained execution/planning
objects and may be hidden or collapsed by default in Desktop.

The distinction is semantic, not a second storage system. Goals and Work Units belong to the same
Work Graph, repository, identity model, and dependency semantics.

### 5.4 Goal model

A Goal should be able to carry at least:

- stable identity;
- title and description;
- status;
- priority/scheduling hint where needed;
- containment parent;
- dependency edges;
- acceptance criteria;
- user notes;
- artifact references;
- evidence references;
- blockers;
- execution summary;
- revision identity.

Suggested lifecycle states include:

```text
Proposed
Ready
InProgress
Blocked
NeedsInput
EvaluationPending
Evaluating
Rework
Completed
Skipped
Cancelled
```

`Completed` is special: executor code must not be able to set it directly.

### 5.5 Work Unit model

Work Units preserve the useful semantics of the existing Todo system:

- title and optional detail;
- `todo`/ready, in-progress, completed, blocked-like execution state;
- priority;
- tags;
- turn/runtime assignment where useful;
- containment under a Goal or another bounded work node;
- dependency edges;
- idempotent mutation behavior;
- batch mutation.

A Work Unit can be completed by the executor because it represents an execution step, not an
independent claim that the user outcome is satisfied.

## 6. Acceptance Criteria Are First-Class Goal State

A Goal is not complete merely because all of its Work Units are complete. Goal completion is judged
against explicit acceptance criteria.

Acceptance criteria must not remain only `Vec<String>` if the implementation needs to distinguish
machine-verifiable facts from semantic judgment. A candidate first model is:

```rust
pub struct AcceptanceCriterion {
    pub id: AcceptanceCriterionId,
    pub description: String,
    pub kind: AcceptanceKind,
    pub required: bool,
}

pub enum AcceptanceKind {
    Invariant,
    Validation,
    Artifact,
    Judgment,
}
```

Examples:

- **Invariant:** credentials never appear in displayed logs;
- **Validation:** the relevant test profile passes;
- **Artifact:** a migration guide must exist;
- **Judgment:** the UX preserves the intended provider-configuration mental model.

The distinction prevents Talos from using an LLM to judge facts that a deterministic validator can
establish more reliably.

## 7. Execution Baseline And Plan Mutation Policy

After shaping, the user explicitly confirms an **Execution Baseline**. The baseline is a revisioned
snapshot of the agreed Mission/Goal outcome and acceptance contract.

During execution, Talos may discover new work. The executor must be allowed to change **how** a
Goal is completed without silently changing **what** is considered completion.

Suggested mutation policy:

### Executor may autonomously

- create or change Work Units inside the current Goal;
- reorder internal work where dependencies permit;
- attach execution summaries and artifact/evidence references;
- mark Work Units in progress/completed/blocked.

### Executor may only through bounded policy

- create a child Goal that does not change the externally visible Mission scope;
- refine implementation detail without weakening an acceptance criterion.

### Executor must request a plan change

- change a top-level Goal;
- remove or weaken required acceptance criteria;
- expand Mission scope;
- change a security or permission boundary;
- alter the final delivery contract.

A scope-changing mutation should produce a structured `PlanChangeProposed` fact that Desktop can
present for accept/edit/reject. The accepted change creates a new baseline revision.

## 8. Todo Evolution: Replace The Domain, Preserve The Useful Semantics

Talos already has a substantial session-scoped Todo implementation. It is not a trivial checklist:
it includes durable SQLite state, statuses, priority, turn assignment, tags, dependency edges, cycle
rejection, idempotent/batch mutation, permission-gated Agent tools, read-only user projections, and
prompt integration.

Creating an independent Desktop Goal store beside that Todo DAG would produce two planning sources
of truth. That is explicitly rejected by this design.

The long-term direction is:

```text
existing Todo domain
        |
        v
canonical Work Graph
        |
        +-- Goal
        \-- WorkUnit
```

Existing Todo semantics map naturally to `WorkUnit`. The migration should preserve:

- UUID/stable identity;
- status and priority semantics where applicable;
- dependency/cycle invariants;
- idempotent create/retry behavior;
- batch mutation;
- permission gating;
- query/filter projections;
- prompt budgeting behavior.

Existing `/todo` and `todo_*` surfaces may remain during a compatibility window, but they must
become projections/adapters over the canonical Work Graph rather than a second repository.

The future canonical mutation surface should be work-oriented (`work_*` or an equivalent reviewed
name), not duplicated `todo_*` plus `goal_*` APIs.

## 9. Execution Experience

### 9.1 Default execution view

The Desktop execution view should emphasize current state, not the raw Agent transcript:

```text
+----------------+---------------------------+------------------+
| GOALS          | CURRENT WORK              | CHANGES/ARTIFACTS|
|                |                           |                  |
| ✓ Architecture | Goal 4: GPUI interface    | M config.rs      |
| ✓ UX           |                           | A provider.rs    |
| ✓ Host API     | Current work:             | A settings.rs    |
| ● GPUI UI      | Build credential form     |                  |
| ○ Integration  |                           | +184 / -21       |
| ○ Verify       | Recent activity:          | [Review Diff]    |
| ○ Delivery     | validation passed ...     |                  |
+----------------+---------------------------+------------------+
```

The default UI should not dump every read/search/tool/stdout event into the main view.

### 9.2 Semantic activity stream

The continuously scrolling execution summary should be derived from structured events such as:

```text
GoalStarted
WorkUnitStarted
DecisionRecorded
ArtifactCreated
ArtifactModified
ValidationStarted
ValidationPassed
ValidationFailed
GoalBlocked
CompletionClaimSubmitted
EvaluationStarted
EvaluationFinding
GoalCompleted
PlanChangeProposed
```

A short human summary can be generated where needed, but the timeline should be grounded in actual
runtime/work events rather than repeated model-authored summaries that can drift from reality.

Detailed logs remain available through drill-down and may include tool calls, stdout/stderr,
provider diagnostics, validation records, and raw runtime events subject to existing redaction and
credential-display rules.

### 9.3 Progress display

Avoid fake precision such as `57%` when the remaining work is inherently uncertain. Prefer:

- `4 / 7 Goals completed`;
- current Goal and Work Unit;
- current phase;
- critical path/dependency projection where useful;
- validation/evaluation state.

## 10. Artifact And Change Workspace

The right-side Desktop review surface should be broader than a `git diff` panel. The canonical
concept is **Artifact**, with changed files being one artifact renderer.

Potential artifact kinds include:

- file;
- patch/change set;
- Git commit;
- document/report;
- image/diagram;
- dataset;
- build output/binary;
- test/validation result;
- URL;
- PR;
- release package.

For coding Missions, the UI should link changes back to the Goal/Work Unit that produced them and
allow the user to answer questions such as:

> Why was this file changed?

through durable traceability:

```text
Goal -> decision/work event -> artifact change -> validation/evaluation evidence
```

Artifact and Evidence should remain their own domain objects or references. Do not turn every file,
finding, decision, or validation record into a Work Graph node and accidentally create a universal
knowledge graph.

## 11. Independent Completion Evaluation

### 11.1 Core rule

The executor may claim that work is ready for evaluation, but it must not self-certify Goal
completion.

```text
Executor -> CompletionClaim -> Evaluator -> Verdict -> Coordinator -> Goal state
```

There must be no direct authority edge:

```text
Executor -> Goal::Completed
```

### 11.2 Completion Claim

When the executor believes a Goal is satisfied, it submits a structured claim such as:

```rust
pub struct CompletionClaim {
    pub goal_id: GoalId,
    pub goal_revision: GoalRevision,
    pub workspace_revision: WorkspaceRevision,
    pub changed_artifacts: Vec<ArtifactRef>,
    pub claimed_evidence: Vec<EvidenceRef>,
    pub executor_summary: String,
}
```

The executor summary is context/hint only. It is neither independent evidence nor the final verdict.

### 11.3 Evaluator independence

The Evaluator must be independent in ways that are enforceable by runtime/product design rather
than only prompt wording:

1. **Separate runtime/Agent instance.** Evaluation is not another turn in the executor's active
   context.
2. **Fresh context.** The evaluator receives Goal definition, baseline, acceptance criteria,
   workspace/artifacts/diff, and relevant evidence. It does not inherit executor reasoning or the
   full executor transcript by default.
3. **Independent inspection.** Executor claims such as “all tests pass” are not trusted without
   evidence or evaluator re-validation.
4. **Read-only by default.** The evaluator may inspect and validate but should not edit the work it
   is judging. Findings return to a rework loop.
5. **Revision binding.** A verdict applies only to the exact Goal/workspace subject that was
   evaluated.

### 11.4 Validator is not Evaluator

Talos already has a shared internal validation service that can produce structured validation
records for internal checks and host-tool adapters. That service is an **evidence producer**.

The new evaluator is the **judgment layer** that maps evidence and direct inspection against Goal
acceptance criteria.

```text
Validator(s) ---> Evidence ---\
                              +--> Evaluator --> Goal Verdict
Artifacts/diff/inspection ----/
```

Machine-verifiable acceptance should prefer validator evidence. Semantic acceptance uses evaluator
judgment.

### 11.5 Evaluation report

Evaluation should be criterion-granular, not only a single PASS/FAIL string:

```rust
pub enum CriterionVerdict {
    Pass,
    Fail,
    Inconclusive,
}

pub struct CriterionEvaluation {
    pub criterion_id: AcceptanceCriterionId,
    pub verdict: CriterionVerdict,
    pub explanation: String,
    pub evidence_refs: Vec<EvidenceRef>,
}

pub struct EvaluationReport {
    pub subject: EvaluationSubject,
    pub overall: CriterionVerdict,
    pub criteria: Vec<CriterionEvaluation>,
    pub findings: Vec<EvaluationFinding>,
}
```

The Desktop can then show:

```text
✓ A1 Provider can be configured
✓ A2 Credential can be validated
✗ A3 Secrets never appear in logs
? A4 Runtime refresh has insufficient evidence
```

### 11.6 Revision binding and stale verdicts

Evaluation approval must follow exact-subject semantics similar to exact-head review discipline.
An evaluation is bound to at least:

- Mission baseline revision;
- Goal revision;
- workspace/change-set revision.

For Git workspaces, the subject may include HEAD plus a deterministic dirty-tree/change-set digest.
If the evaluated subject changes after a PASS, the previous evaluation becomes **stale** and cannot
continue to authorize `Completed` or `Delivered` state.

### 11.7 Rework loop

A failing evaluation produces structured findings and transitions the Goal into rework. The
executor receives the Goal contract plus evaluator findings, performs new work, and submits a new
Completion Claim for a new exact revision.

### 11.8 Mission-level evaluation

`all(goals.completed)` is not sufficient to prove a Mission is complete. Individually correct
Goals can fail when integrated.

After all required Goal evaluations pass, the Mission must support a final independent evaluation
against Mission-level outcomes and cross-Goal integration criteria before delivery.

## 12. Delivery As A Durable Product Object

The end of a Mission should not be an assistant message saying “done.” It should produce a durable
**Delivery** assembled from evaluated facts:

```text
Mission baseline
+ completed Goal Graph
+ Goal evaluation reports
+ final Mission evaluation
+ artifacts/change set
+ validation evidence
+ final workspace revision
= Delivery
```

A coding Delivery can render:

- outcome summary;
- Goal completion and evaluation status;
- acceptance criteria with evidence;
- changed files/diff;
- validation/test/build results;
- security findings or unresolved warnings;
- screenshots or runtime proof where applicable;
- deviations from the baseline;
- commit/PR actions when the change set is ready.

A non-coding Delivery may instead contain research reports, datasets, documents, diagrams, source
coverage, decisions, or other domain artifacts.

The important invariant is that check marks in Delivery are derived from evaluable state and
records, not from executor prose.

## 13. Shared Domain Versus Desktop UX

The following distinction is architectural:

### Shared Talos domain

- Mission identity/lifecycle;
- Work Graph;
- Goal/WorkUnit semantics;
- acceptance criteria;
- execution baseline and mutation policy;
- Completion Claim;
- evaluation subject/report/finding;
- exact-revision/staleness rules;
- artifact/evidence references.

### Desktop product UX

- Goal tree/graph visual projection;
- visual shaping and direct manipulation;
- current-work execution view;
- semantic activity timeline;
- artifact/change workspace;
- evaluation progress and findings UI;
- Delivery review UI;
- GPUI window/platform integration.

The Goal-first experience is a Desktop differentiator, but the Work Graph and evaluation semantics
must not be implemented as GPUI-local state.

TUI may continue to expose a compact conversation-first projection over the same canonical work
state, including a compatibility `/todo` view during migration.

## 14. Future Separate Desktop Prerequisite Implementation PR

This section records the **future action**. It is intentionally not executed by this documentation
PR.

Before creating the first GPUI Desktop implementation PR, create a **separate prerequisite
implementation PR** whose purpose is to establish the shared work/evaluation foundation that
Desktop will consume.

Suggested PR intent/title:

> `foundation: introduce canonical work graph and independent goal evaluation`

Normal requirement-intake, iteration selection, ADR, Collaboration Claim, and review rules remain
mandatory before that implementation branch is created.

### 14.1 Required implementation actions

The prerequisite PR should:

1. **Establish the canonical work domain.**
   - Prefer a dedicated `crates/talos-work/` if the implementation confirms that Mission lifecycle
     is no longer session-owned and the crate preserves a single responsibility.
   - Define Mission, Work Graph, Goal, WorkUnit, containment, dependency, revision, and acceptance
     criterion types.
   - Do not create a general workflow/scheduler engine.

2. **Evolve Todo into Work Graph compatibility.**
   - Migrate/reuse current Todo persistence and invariants where practical.
   - Map legacy Todo items to Work Units.
   - Preserve dependency cycle detection, idempotency, batch mutation, permission checks, and query
     behavior.
   - Keep `/todo` and `todo_*` only as explicitly bounded compatibility adapters if needed.
   - Do not retain a parallel Todo repository as a second source of truth.

3. **Introduce Goal authority rules.**
   - Executor mutation APIs must not permit direct `Goal -> Completed` transitions.
   - Work Units remain executor-completable.
   - Goal completion is coordinator-owned after a current independent PASS.

4. **Introduce Completion Claims.**
   - Executor submits an exact-revision evaluation request rather than self-certifying completion.

5. **Introduce independent Evaluation models and orchestration boundary.**
   - Separate evaluator runtime/session from executor runtime/session.
   - Fresh evaluator context by default.
   - Read-only evaluator policy by default.
   - Criterion-granular PASS/FAIL/INCONCLUSIVE reports and findings.
   - Rework loop support.

6. **Bind evaluation to exact revisions.**
   - Define Mission/Goal/workspace subject identity.
   - Invalidate or mark stale any report whose subject changes.

7. **Reuse existing Validation Service as evidence input.**
   - Do not conflate validation execution with Goal evaluation.
   - Allow machine-verifiable acceptance criteria to reference structured validation records.

8. **Reserve Mission-level evaluation.**
   - A Mission must not become delivery-ready solely because every child Goal is completed.
   - Support a final cross-Goal/integration evaluation subject and verdict.

9. **Provide runtime/product-neutral projections.**
   - Expose enough typed state/events for TUI and the future Desktop to consume without depending on
     GPUI or terminal rendering.

10. **Provide migration and regression tests.**
    - Todo compatibility/data migration;
    - graph invariants and cycle rejection;
    - Goal authority enforcement;
    - stale evaluation after revision change;
    - PASS -> Completed transition;
    - FAIL -> Rework -> new evaluation;
    - evaluator isolation;
    - validation-evidence linkage;
    - Mission-level evaluation gate.

### 14.2 Explicit exclusions for the prerequisite PR

The prerequisite PR must **not**:

- create `talos-desktop` or link GPUI;
- implement Desktop windows, panels, renderer, motion, packaging, or updater;
- reproduce the TUI in a GUI;
- create a generic multi-agent framework solely to support one evaluator role;
- add a general workflow scheduler;
- make Artifact/Evidence every kind of graph node;
- weaken permissions, sandboxing, credential display, or durable-session boundaries;
- claim Desktop is shipped or implementation-ready until its own governed iteration is selected.

### 14.3 Prerequisite PR acceptance

The future prerequisite PR is ready only when:

- there is one canonical durable work-state source of truth;
- Goal and WorkUnit share the Work Graph while keeping different authority semantics;
- current Todo behavior has an explicit migration/compatibility path;
- executor code cannot directly complete a Goal;
- independent evaluator flow exists with a fresh context boundary;
- evaluation reports bind to exact subject revisions and become stale after mutation;
- Validation Service records can be consumed as evidence without being treated as evaluator verdicts;
- failed evaluation creates a deterministic rework loop;
- Mission-level final evaluation is represented;
- TUI/runtime behavior remains compatible through reviewed adapters/projections;
- required locked tests and governance validation pass.

Only after that PR is merged should the first Desktop implementation PR depend on the new canonical
work/evaluation surface.

## 15. Desktop Implementation Sequence After The Prerequisite

The Desktop sequence should then proceed independently:

### D0 — Desktop architecture activation

- reconcile the GPUI host decision with current repository ADR requirements;
- select the implementation iteration and effective Collaboration Claim;
- define Desktop process/runtime topology and packaging boundary;
- confirm renderer dependency/security review.

### D1 — GPUI skeleton

- create the product package/crate;
- open a native window;
- establish typed Desktop command/event wiring;
- no attempt at broad feature parity.

### D2 — Mission shaping vertical slice

- create/open a Mission;
- display Goal tree projection;
- edit Goal and acceptance state;
- create an Execution Baseline.

### D3 — Execution vertical slice

- run one Mission against Talos runtime;
- display current Goal/Work Unit;
- semantic activity stream;
- interrupt/pause and plan-change path;
- artifact/change panel.

### D4 — Independent evaluation vertical slice

- completion claim;
- evaluator launch;
- criterion-by-criterion progress/findings;
- rework loop;
- stale-evaluation handling.

### D5 — Delivery vertical slice

- final Mission evaluation;
- evaluated Delivery object;
- coding change review and later commit/PR actions;
- non-coding artifact rendering where applicable.

Platform features such as tray, notifications, drag/drop, multi-window, updater, signing, and
packaging follow after the core Mission workflow is proven.

## 16. Non-Goals Of This Design Baseline

This document does not authorize:

- implementation of `talos-work`;
- Todo schema migration;
- creation of `talos-desktop`;
- GPUI dependency adoption;
- a generic multi-agent orchestrator;
- automatic approval of plan changes;
- evaluator write access;
- remote/multi-user Mission collaboration;
- a universal artifact/knowledge graph;
- release or packaging changes.

Those changes require their own governed implementation work.

## 17. Open Questions For The Prerequisite/ADR Phase

The following details remain implementation/ADR questions rather than reasons to reopen the product
model:

- final public naming: `talos-work`, Work Graph types, and `work_*` tools;
- persistence placement and migration mechanics from `todos.sqlite`;
- exact WorkspaceRevision representation for Git and non-Git workspaces;
- whether evaluator provider/model selection is caller-configurable or uses a product policy;
- exact evaluator read/validation tool profile;
- whether child-Goal creation inside an Execution Baseline is always approval-gated or can be
  policy-classified as scope-neutral;
- how much of the existing `talos-conversation` projection can serve the second renderer before a
  new presentation/text crate is justified;
- exact Artifact/Evidence storage model beyond references needed by the first vertical slice.

## 18. Decision Summary

This design baseline records the following direction:

1. Talos Desktop is **not** a graphical TUI clone.
2. Desktop is a **Goal-first Mission workspace**; TUI remains conversation-first.
3. GPUI is the selected Desktop renderer direction; Tauri/WebView is not the current route.
4. The canonical work model is one DAG-capable **Work Graph** with Goal and WorkUnit node semantics.
5. The existing Todo domain should evolve into WorkUnit compatibility rather than coexist as a
   second planning source of truth.
6. Acceptance Criteria are first-class Goal state.
7. Execution Baselines and Plan Mutation Policy distinguish “how to do it” from “what counts as
   done.”
8. Executors may complete Work Units but may only submit **Completion Claims** for Goals.
9. Goal completion requires an **independent evaluator** with fresh context, read-only defaults,
   criterion-level verdicts, and exact-revision binding.
10. Existing Validation Service produces evidence; it does not replace independent evaluation.
11. A final Mission-level evaluation is required before delivery.
12. Default execution UX shows Goal/current-work state, semantic activity, and artifact changes;
    detailed raw logs are drill-down material.
13. Delivery is a durable evaluated object, not a final assistant message.
14. The shared Work Graph/evaluation foundation must land in a **future separate prerequisite PR**
    before GPUI Desktop implementation begins.
