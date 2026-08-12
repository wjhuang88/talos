# Talos Desktop Goal-Oriented Workspace Design Baseline

> Status: Design baseline / proposal refinement — **no implementation authorization**
>
> Date: 2026-08-12
>
> Owner direction: `DESKTOP-001`
>
> Source discussion: maintainer product/architecture discussion consolidating the Desktop renderer,
> goal-oriented workflow, Todo evolution, independent evaluation, artifact review, delivery model,
> visual direction, and initial internationalization requirement.

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

The Desktop visual and internationalization baselines are additionally defined in:

- `docs/design/talos-desktop/DESIGN.md`
- `docs/design/talos-desktop/I18N.md`

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

## 4. Internationalization Is A First-Class Desktop Requirement

The first visible GPUI Desktop implementation must be internationalized from the beginning.
Internationalization is not deferred polish and must not require later rewriting view code to remove
hard-coded labels.

Initial supported UI locales are:

- Simplified Chinese (`zh-CN`);
- English (`en-US`).

Locale is a Desktop client/presentation preference. It must not become part of Mission, Goal,
WorkUnit, Execution Baseline, Completion Claim, Evaluation, Evidence, Artifact, or Delivery identity.
Changing the UI language therefore does not create a new work revision or make an evaluation stale.

Product-controlled UI strings are localized through a catalog/key abstraction or equivalent typed
message interface. User-authored Mission/Goal content, code, paths, commands, raw logs, external
content, Artifacts, and raw evidence are not silently translated.

The first GPUI visual/interaction slice must validate the same Execution experience in both initial
languages, including:

- navigation and controls;
- Current Goal and Current Work hierarchy;
- Mission path;
- Recent Activity;
- change summary;
- one blocked/error state;
- one evaluation state;
- layout/wrapping at normal laptop width;
- mixed CJK/Latin rendering;
- Chinese IME in editable controls;
- system-language negotiation, explicit language selection, persistence, and deterministic fallback.

Detailed localization rules are authoritative in `docs/design/talos-desktop/I18N.md`.

## 5. Mission Lifecycle

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

## 6. Work Graph: One Canonical Planning And Execution Model

### 6.1 Why a graph

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

### 6.2 Mission

A Mission is the durable unit of delegated outcome. It can span multiple runtime/session instances,
including executor sessions, evaluator sessions, rework sessions, and later reconnects.

A Mission therefore must not be modeled as a child object whose lifetime is owned by one transcript
session.

### 6.3 Work nodes

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

### 6.4 Goal model

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

### 6.5 Work Unit model

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

## 7. Acceptance Criteria Are First-Class Goal State

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

## 8. Execution Baseline And Plan Mutation Policy

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

## 9. Todo Evolution: Replace The Domain, Preserve The Useful Semantics

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

## 10. Execution Experience

### 10.1 Default execution view

The Desktop execution view should emphasize current state, not the raw Agent transcript. The visual
baseline is light-first, Nord-derived, low-density, and organized around one dominant execution
narrative. See `docs/design/talos-desktop/DESIGN.md`.

The default UI should not dump every read/search/tool/stdout event into the main view.

### 10.2 Semantic activity stream

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

### 10.3 Progress display

Avoid fake precision such as `57%` when the remaining work is inherently uncertain. Prefer:

- `4 / 7 Goals completed`;
- current Goal and Work Unit;
- current phase;
- critical path/dependency projection where useful;
- validation/evaluation state.

## 11. Artifact And Change Workspace

The Desktop review surface should be broader than a `git diff` panel. The canonical concept is
**Artifact**, with changed files being one artifact renderer.

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

## 12. Independent Completion Evaluation

### 12.1 Core rule

The executor may claim that work is ready for evaluation, but it must not self-certify Goal
completion.

```text
Executor -> CompletionClaim -> Evaluator -> Verdict -> Coordinator -> Goal state
```

There must be no direct authority edge:

```text
Executor -> Goal::Completed
```

### 12.2 Completion Claim

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

### 12.3 Evaluator independence

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

### 12.4 Validator is not Evaluator

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

### 12.5 Evaluation report

Evaluation should be criterion-granular, not only a single PASS/FAIL string:

```rust
pub enum CriterionVerdict {
    Pass,
    Fail,
    Inconclusive,
}
```

Each criterion result should retain evidence references and findings sufficient for a user or
executor to understand why it passed, failed, or could not be determined.

### 12.6 Exact-revision binding

Evaluation applies to an explicit subject revision, for example:

```text
Mission revision
+ Goal revision
+ workspace/content revision
```

For Git-backed coding work this may include the exact HEAD plus a dirty-worktree digest or an
equivalent stable content identity. The concrete scheme requires implementation design.

If relevant subject state changes after PASS, that verdict becomes stale rather than silently
continuing to certify the modified work.

Locale-only presentation changes are explicitly excluded from the evaluation subject. Switching
`zh-CN`/`en-US` does not stale an otherwise unchanged evaluation.

### 12.7 Rework

A failed Goal evaluation returns structured findings to the executor:

```text
Evaluation FAIL
      |
      v
Rework
      |
      v
Executor changes subject revision
      |
      v
new CompletionClaim
      |
      v
fresh Evaluation
```

An evaluator should not fix the work it evaluates by default; otherwise executor/evaluator authority
collapses back into one role.

## 13. Mission-Level Evaluation

All child Goals passing does not prove the integrated Mission outcome.

After required Goal evaluations pass, a separate Mission-level evaluation should verify the final
integrated state against Mission-level acceptance and cross-Goal behavior before Delivery.

```text
all required Goals evaluated PASS
              |
              v
      Mission Evaluation
              |
        PASS / FAIL
          |       |
          |       +--> Rework
          v
       Delivery
```

Mission evaluation may reuse Goal evidence, but it must be able to add integration-level validation
and judgment.

## 14. Delivery Is A Durable Evaluated Object

Completion should not end with an assistant message saying “done.”

A Delivery is generated from evaluated work state and should summarize:

- Mission outcome;
- Goal completion state;
- acceptance/evaluation results;
- validation evidence;
- Artifacts/change set;
- unresolved warnings or deviations;
- exact delivered workspace/revision identity.

For coding work, Delivery may expose actions such as:

```text
Review Changes
Create Commit
Create PR
Open Build/Artifact
```

Those actions remain governed by existing permission and Git workflow policies.

The Delivery summary itself is localized presentation. The delivered Artifact and Evidence facts
remain canonical and are not rewritten when the UI locale changes.

## 15. Desktop And TUI Do Not Need Feature-Shape Parity

The shared Work Graph can be projected differently by each product.

Desktop may expose Goal shaping, graph/outline views, drag/reorder, rich Evaluation, Artifact review,
and Delivery surfaces.

TUI can keep a compact projection such as:

```text
Mission: Add provider settings

Goals
✓ Architecture
● Provider configuration
○ Verification

Current work
✓ inspect schema
● implement form
○ run tests
```

This does not require TUI to reproduce Desktop layout or localization behavior. Desktop
internationalization is not a requirement to retrofit the current TUI in the same implementation
slice.

## 16. Shared Domain, Desktop-Specific Interaction

Goal-first is a Desktop interaction paradigm, but Work Graph and Evaluation should be shared domain
capabilities rather than GPUI-only models.

Likewise, locale is a Desktop client presentation concern and must not leak into shared Work Graph or
Evaluation state.

The target dependency direction is conceptually:

```text
shared work/runtime domain
          |
          v
UI-neutral projection/controller
       /             \
      /               \
   TUI               Desktop
                      GPUI
                       |
                locale projection
                zh-CN / en-US
```

Do not create new shared crates purely to make this diagram exact. Existing `talos-conversation`,
`talos-runtime`, and `talos-session` responsibilities must be reconciled first.

## 17. Candidate Shared Work Crate

The current Todo implementation is session-owned. Mission/Goal/Evaluation lifetime is expected to
span executor, evaluator, rework, and later reconnect sessions. A dedicated durable work-domain crate
such as `talos-work` is therefore a credible candidate.

That name and boundary are **not authorized merely by this proposal**. The future prerequisite PR
must confirm dependency direction and responsibility before adding the crate.

If created, its responsibility should be narrowly defined around durable work state:

```text
Mission
Work Graph
Goal / WorkUnit
Acceptance Criteria
Graph mutation policy
Completion Claims
Evaluation models
Persistence
```

It should not become a generic workflow engine, generic scheduler, GUI framework, or multi-agent
framework.

Localization catalogs and Desktop language preferences do **not** belong in this shared work crate.

## 18. Current Repository Reconciliation

This design supersedes the older Tauri-oriented recommendation in `docs/proposals/talos-desktop.md`
while preserving Issue #29 and `DESKTOP-001` as historical/governance context.

Current architecture must be respected:

- `talos-runtime` is the supported reusable SDK facade;
- `talos-conversation` already provides UI-independent product projection and must be evaluated
  before inventing an overlapping presentation layer;
- durable session/transcript ownership remains in current session/runtime boundaries;
- SESSION-009 owns future attach/detach/replay/multi-client semantics;
- client viewport/layout state remains client-owned;
- existing Validation Service is reusable evidence infrastructure.

The earlier proposed `talos-presentation` crate should therefore **not** be created automatically.
The Desktop implementation is the second real frontend and may justify extraction of currently
CLI/TUI-owned orchestration, but only where actual dependency evidence requires it.

Similarly, an earlier `talos-motion` idea should begin as Desktop-local motion semantics unless a
second consumer demonstrates a need for a shared crate.

Desktop localization is a real first-client requirement, but no shared `talos-i18n` crate should be
invented without evidence that a second product needs the same abstraction.

## 19. SESSION-009 Dependency Boundary

The first Desktop vertical slice can use a local, embedded, single-client runtime topology. It does
not need to wait for every future SESSION-009 multi-client/reconnect capability.

SESSION-009 becomes a hard dependency for behavior such as:

- separate daemon/session ownership;
- attach/detach;
- reconnect;
- multi-window concurrent clients;
- observer/controller fanout;
- replay after connection loss.

The first Desktop slice must not invent alternate session ownership merely to bypass SESSION-009.

## 20. Future Separate Desktop Prerequisite Implementation PR

This document defines the required **future action**. The documentation PR containing this proposal
must not create the implementation PR itself.

Before the first GPUI Desktop implementation PR, create a separate governed prerequisite
implementation PR after normal requirement intake, ADR/migration review, iteration selection, and an
effective Collaboration Claim.

A suitable conceptual title is:

> `foundation: introduce canonical work graph and independent goal evaluation`

The title is illustrative; governance/story IDs and final scope must come from the selected
iteration.

### 20.1 Required implementation actions

The future prerequisite PR should, subject to the selected iteration and review, establish:

1. **Canonical work domain**
   - confirm whether `talos-work` is the correct crate boundary;
   - add Mission, Work Graph, Goal, WorkUnit, containment, dependency, and stable identity;
   - define revision semantics required by evaluation.

2. **Todo migration/adaptation**
   - preserve current Todo status/priority/tag/dependency/idempotency/batch semantics as WorkUnit
     behavior where applicable;
   - preserve cycle rejection and permission gating;
   - provide a compatibility projection/adaptor for `/todo` and `todo_*` during migration;
   - avoid maintaining a second Todo repository beside Work Graph.

3. **Acceptance Criteria**
   - make acceptance first-class Goal state;
   - distinguish deterministic validation/artifact/invariant checks from semantic judgment where
     useful.

4. **Goal authority**
   - executor cannot directly transition Goal to `Completed`;
   - executor can complete Work Units;
   - executor can request Goal evaluation through a Completion Claim.

5. **Completion Claim / Evaluation model**
   - add structured Completion Claim;
   - add Evaluation request/subject/report/finding/criterion-verdict models;
   - bind verdicts to exact subject revisions;
   - stale verdicts after relevant mutation.

6. **Independent evaluator boundary**
   - separate Agent/runtime instance or equivalent enforced execution boundary;
   - fresh evaluation context;
   - read-only by default;
   - no inheritance of executor reasoning as evaluation truth.

7. **Validation evidence integration**
   - consume existing Validation Service records as Evidence;
   - do not make Validator itself the Goal judgment authority;
   - deterministic acceptance prefers deterministic evidence.

8. **Rework loop**
   - FAIL/INCONCLUSIVE are representable without pretending Goal completion;
   - a changed subject requires a new Completion Claim and fresh Evaluation.

9. **Mission final-evaluation contract**
   - define the Mission-level gate so all Goal PASS results do not automatically imply Delivery.

10. **Projection/API boundary**
    - expose enough UI-neutral state/events for later Desktop use;
    - do not introduce GPUI types into shared crates;
    - reconcile `talos-conversation` and current CLI `tui_bridge` orchestration before creating new
      presentation abstractions.

11. **Migration/regression tests**
    - existing Todo behavior remains usable through the compatibility surface;
    - cycle/idempotency/batch behavior is preserved;
    - executor cannot self-certify Goal completion;
    - evaluation staleness is tested;
    - locale is not part of this shared work-domain PR.

### 20.2 Required acceptance

The prerequisite PR should not be considered complete unless:

- there is one canonical work-state source of truth;
- Todo no longer needs to evolve as an independent parallel planning domain;
- existing Todo data/behavior has a defined migration or compatibility path;
- Goal and WorkUnit authority differs as documented;
- Completion Claim -> Evaluation -> verdict -> Goal transition is demonstrable;
- evaluator context/authority is independent from executor context/authority;
- evaluation is criterion-granular and revision-bound;
- Validation Service evidence can be consumed without conflating Validator and Evaluator;
- rework after FAIL produces a new evaluation subject;
- Mission-level final evaluation is represented;
- TUI compatibility does not require depending on Desktop/GPUI;
- no Desktop UI is claimed as implemented.

### 20.3 Explicit exclusions from the prerequisite PR

The prerequisite PR must **not**:

- create `talos-desktop`;
- add GPUI;
- implement Desktop windows/panels/components;
- create Desktop i18n catalogs or locale settings;
- package/sign/update a Desktop application;
- implement SESSION-009 remote/multi-client behavior unless separately selected;
- turn Evaluation into a generic unrestricted multi-agent framework;
- make evaluator write access the default;
- make every Artifact/Evidence/Decision a Work Graph node;
- delete compatibility Todo surfaces without an explicit migration acceptance;
- weaken permission, sandbox, credential, transcript, or evidence boundaries.

### 20.4 Separate first GPUI Desktop implementation

Only after the prerequisite work has merged and passed independent review should a separately
selected/claimed Desktop implementation slice begin.

That first GPUI slice should establish:

- `talos-desktop` product host and GPUI dependency boundary;
- the selected Execution-page vertical slice;
- the visual baseline from `docs/design/talos-desktop/DESIGN.md`;
- localization infrastructure and complete `zh-CN` / `en-US` coverage for the selected visible
  slice;
- system/explicit locale selection and persistence;
- bilingual layout validation and Chinese IME;
- mocks where appropriate before full runtime binding;
- then real runtime/work/evaluation binding through the shared APIs.

The first GPUI slice should not ship a hard-coded single-language prototype that must later be
structurally internationalized.

## 21. Development Phases After The Prerequisite

Subject to future governed iteration selection, the broad Desktop sequence remains:

```text
D0  GPUI dependency/ADR and repository reconciliation
D1  shared text semantics only where current TUI ownership blocks reuse
D2  UI-neutral product/controller extraction only where second-consumer evidence requires it
D3  GPUI execution skeleton + bilingual visual/IME validation
D4  real Runtime/Work Graph vertical slice
D5  approvals/permission interaction
D6  durable Mission/session behavior
D7  Goal shaping + Evaluation + Artifact/Delivery review
D8  packaging/release/platform integration
```

These phases are planning guidance, not selected iterations.

## 22. Success Criteria For The Product Direction

The Desktop direction is successful when:

- users can state a rough outcome and collaboratively shape a verifiable work plan;
- the agreed plan becomes revisioned product state rather than transcript-only context;
- users can understand current execution without reading raw tool logs;
- Work Graph is the only planning/execution source of truth rather than competing Goal/Todo stores;
- executor autonomy covers how work is done without silent scope/acceptance mutation;
- executor cannot self-certify Goal completion;
- independent evaluation produces criterion-level, evidence-backed, exact-revision verdicts;
- integrated Mission completion receives its own final evaluation;
- Artifact/change traceability explains what changed and why;
- Delivery represents evaluated output rather than an assistant's closing message;
- TUI remains useful without copying Desktop UX;
- Desktop remains a GPUI host above existing Talos runtime/security boundaries;
- the initial Desktop user-visible surface works coherently in both Simplified Chinese and English;
- changing UI locale never changes canonical work/evaluation identity.

## 23. Non-Goals Of This Proposal

This proposal does not authorize:

- production Desktop code;
- a GPUI dependency before the required implementation governance/ADR path;
- a new presentation crate by name;
- a generic workflow engine;
- a generic multi-agent framework;
- full SESSION-009 implementation;
- immediate removal of Todo compatibility surfaces;
- automatic translation of user-authored/external/code/evidence content;
- initial Desktop locale coverage beyond `zh-CN` and `en-US`;
- a claim that any Desktop behavior is currently shipped.

## 24. Required Reads For Future Implementers

Before the prerequisite implementation PR:

- `docs/backlog/active/DESKTOP-001-desktop-product-direction.md`
- `docs/backlog/active/TODO-001-session-todo-list.md`
- `docs/backlog/active/TODO-002-todo-mutation-reliability.md`
- `docs/backlog/active/VALIDATION-001-internal-validation-service.md`
- `docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md`
- `docs/backlog/active/SESSION-009-multi-client-session-architecture.md`
- `docs/decisions/042-embedded-durable-runtime-session-boundary.md`
- `docs/decisions/052-sdk-publication-and-composition-boundary.md`
- current `talos-session` Todo implementation;
- current `talos-conversation` projection;
- current `talos-runtime` facade;
- current CLI/TUI bridge/orchestration.

Before the first GPUI Desktop implementation PR, additionally read:

- `docs/design/talos-desktop/DESIGN.md`
- `docs/design/talos-desktop/I18N.md`
- then-current GPUI and Rust localization/platform-text evidence.

## 25. Governance Note

This is a design/proposal baseline. It does not create an implementation iteration, Collaboration
Claim, crate, migration, Desktop binary, localization catalog, or PR authorization.

The required next implementation action remains a **future separate prerequisite PR**, created only
after normal repository governance selects and authorizes that work. Internationalization belongs to
the later first GPUI Desktop implementation slice, not to the shared Work Graph/evaluation
prerequisite.
