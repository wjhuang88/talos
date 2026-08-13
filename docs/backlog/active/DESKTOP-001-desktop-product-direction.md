# DESKTOP-001: Desktop Product Direction And Technology Boundary

| Field | Value |
|---|---|
| Story ID | DESKTOP-001 |
| Type | Product / Architecture Spike |
| Priority | P3 |
| Status | Deferred — refined design baseline retained; no implementation iteration selected |
| Source | [GitHub Issue #29](https://github.com/wjhuang88/talos/issues/29) |
| Selected Iteration | None |
| Depends On | RUNTIME-001 reusable runtime API; Work Graph/evaluation prerequisite; SESSION-009 for later multi-client behavior; permission and distribution decisions |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #29 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-12 |
| Handoff / Release Condition | Establish an effective claim and select the prerequisite/desktop iterations before implementation. |

## Identity / Goal / Value

Preserve and refine the Desktop product direction without implying that Desktop or its prerequisite
runtime/domain changes are authorized for implementation.

Talos Desktop is not intended to be a graphical reproduction of the TUI. The refined product
position is a **goal-oriented Mission workspace** for shaping outcomes, supervising execution,
reviewing artifact changes, independently evaluating Goal completion, and receiving a durable
Delivery.

The TUI remains conversation-first. Desktop is goal-first and state-centric.

## Refined Design Baseline (2026-08-12)

The consolidated product and visual design are owned by:

- `docs/proposals/talos-desktop-goal-oriented-workspace.md`
- `docs/design/talos-desktop/DESIGN.md`
- `docs/design/talos-desktop/I18N.md`

Key direction recorded there:

- GPUI is the selected Desktop renderer direction; the old Tauri/WebView recommendation is
  historical context, not the current route.
- Desktop is a host/client surface above existing Talos runtime/security boundaries, not a second
  Agent execution engine.
- Mission -> Work Graph -> Goal/WorkUnit becomes the proposed shared work model.
- Existing Todo semantics should migrate into WorkUnit compatibility instead of remaining a
  parallel planning source of truth.
- Acceptance Criteria, Execution Baselines, and Plan Mutation Policy become first-class work facts.
- Executors may complete Work Units but may not self-certify Goal completion.
- Goal completion requires an independent evaluator with fresh context, read-only defaults,
  criterion-level verdicts, and exact-revision binding.
- Existing Validation Service is an evidence producer, not the evaluator itself.
- Mission-level independent evaluation gates Delivery after Goal-level evaluation.
- Desktop execution UX defaults to current Goal/work state, semantic activity, and artifact/change
  review; detailed raw logs are drill-down material.
- Delivery is a durable evaluated object rather than a final assistant message.
- Desktop is light-first with a Nord-derived visual language and a focused, low-density execution
  experience rather than a dense technical dashboard.
- Internationalization is a first-class Desktop requirement. The initial Desktop release must
  support Simplified Chinese (`zh-CN`) and English (`en-US`) as complete UI locales rather than
  hard-coded per-screen strings.

## Scope

This Story owns the directional product/architecture question and the handoff requirements for
future implementation. It does not itself authorize implementation.

The current refinement covers:

- product split between conversation-first TUI and goal-first Desktop;
- GPUI renderer direction and Rust-first boundary;
- shared Work Graph domain direction;
- Todo evolution/migration direction;
- independent Goal/Mission evaluation contract;
- execution/activity/artifact/Delivery UX model;
- light-first Nord-derived Desktop visual direction;
- internationalization architecture and initial `zh-CN` / `en-US` product requirement;
- definition of a future governed Desktop prerequisite chain.

## Internationalization Requirement

Internationalization must be designed into the first GPUI Desktop slice rather than deferred until
a later polish phase.

Initial supported UI locales:

- Simplified Chinese (`zh-CN`);
- English (`en-US`).

The detailed contract is defined in `docs/design/talos-desktop/I18N.md`. Key architecture rules are:

- locale is Desktop client/presentation state, not Mission/Work Graph/session semantic identity;
- switching locale must not mutate Goal revisions, Execution Baselines, Evaluation subjects,
  Evidence, Artifacts, or Delivery revisions;
- product-controlled user-facing strings use a localization catalog/key interface rather than
  hard-coded English/Chinese strings in view code;
- user-authored Mission/Goal text, code, paths, commands, raw logs, external content, and raw
  evidence are not silently translated;
- locale-sensitive labels, counts, dates/times, errors, accessibility names, and controls are
  localized at the presentation boundary;
- layout must tolerate Chinese/English length and wrapping differences without creating separate
  page structures;
- Chinese IME correctness is an initial release-quality interaction requirement;
- system-locale negotiation plus an explicit persisted language preference must be supported;
- unsupported locales fall back deterministically to English.

The first GPUI visual/interaction spike must exercise the same Execution experience in both
`zh-CN` and `en-US` so layout and hierarchy regressions are discovered before broad Desktop work.

## Future Governed Prerequisite Chain

Before the first GPUI Desktop implementation PR, an ordered set of separately governed and reviewed
implementation slices must establish the shared work/evaluation foundation.

The exact action list, acceptance, and exclusions are documented in
`docs/proposals/talos-desktop-goal-oriented-workspace.md`, section **Future Governed Desktop
Prerequisite Chain**.

The chain is deliberately split so no single PR owns all of the following concerns:

1. P0 — decision and migration contract;
2. P1 — canonical Work Domain and Todo compatibility;
3. P2 — Completion Claim and Evaluation state model;
4. P3 — independent evaluator runtime and validation evidence;
5. P4 — Mission final gate, UI-neutral projection, and end-to-end closure.

Each item requires its own selected iteration, effective Collaboration Claim, implementation PR,
acceptance evidence, and independent review. None may create `talos-desktop`, add GPUI, implement
Desktop windows/panels, or claim Desktop shipment. Desktop internationalization is likewise **not**
part of this shared chain; it belongs to the later GPUI Desktop implementation boundary.

## First GPUI Desktop Implementation Boundary

After the full shared prerequisite chain is merged and a Desktop implementation iteration/claim is
selected, the first GPUI Desktop slice must establish the localization foundation together with the
selected visible surface. It must not ship a hard-coded single-language prototype that would require
later view-level rewrites to internationalize.

At minimum that slice should:

- select and document the concrete Rust/GPUI localization mechanism using then-current ecosystem
  evidence;
- establish stable localization keys/catalog loading and deterministic fallback;
- provide `zh-CN` and `en-US` catalogs for every visible string in the selected slice;
- persist system/explicit language preference as Desktop client state;
- validate Chinese IME for editable controls in scope;
- verify the selected Execution layout in both languages at normal laptop width;
- test that locale switching cannot alter canonical Mission/Goal/Evaluation identity or evidence.

## Exclusions

- No Desktop implementation, GPUI dependency, packaging pipeline, or native window code in this
  Story while it remains Deferred.
- No parallel Desktop-only Goal store beside the existing Todo domain.
- No generic workflow scheduler or generic multi-agent framework solely for the evaluator role.
- No weakening of Rust-first core ownership, permission, credential, sandbox, durable-session, or
  revision/evidence boundaries.
- No assumption that child Goal completion alone proves Mission completion.
- No requirement for the initial Desktop release to support locales beyond `zh-CN` and `en-US`.
- No automatic translation of user-authored, source-code, Artifact, external, or raw evidence
  content as part of the localization system.
- No requirement in this Story to retrofit the whole current CLI/TUI with Desktop localization.

## Dependencies And Existing Foundations

- `RUNTIME-001` reusable runtime API.
- `VALIDATION-001` shared internal validation service as evidence producer.
- Current `talos-session` Todo persistence/tool semantics as migration input.
- Current `talos-conversation` UI-independent projection; reconcile before inventing another
  presentation abstraction.
- `SESSION-009` multi-client model for later attach/detach/reconnect/multi-window behavior. A local
  single-client embedded Desktop vertical slice should not be blocked solely on SESSION-009.
- Permission/sandbox/credential and distribution decisions.
- Platform text/input/font behavior required for English, Chinese, and Chinese IME on macOS,
  Windows, and Linux must be validated at the GPUI implementation boundary.

## Decision Links And Constraints

- Desktop is a host/client surface above `talos-runtime`, not a second agent execution engine.
- GPUI/native dependency implications require normal ADR/security review before implementation.
- Renderer dependencies do not flow into `talos-core` or `talos-runtime`.
- TUI and Desktop are independent renderers; neither depends on the other.
- Work Graph/evaluation semantics are shared Talos domain state, not GPUI-local state.
- Transcript/execution/approval facts remain session/runtime-owned where applicable; visual
  viewport/layout/cursor state remains client-owned.
- Evaluator PASS is bound to an exact evaluation subject and becomes stale after relevant mutation.
- Locale is client-owned presentation preference. Locale changes do not alter canonical Work Graph,
  session, Artifact, Evidence, Evaluation, or Delivery identity.
- Localized display labels must never become protocol values, persistence IDs, command IDs, or
  domain enum identity.
- Multi-client or reconnect behavior must consume SESSION-009 rather than invent connection-owned
  sessions.

## Uncertainty And Validation Path

Resume implementation only through bounded, governed work:

1. run requirement intake for the shared Work Graph/evaluation prerequisite;
2. create required ADR(s) and migration plan for public/breaking boundaries;
3. select an implementation iteration and establish an effective Collaboration Claim;
4. land and independently review each prerequisite-chain slice in order;
5. then select the first GPUI Desktop implementation iteration and claim;
6. in that Desktop slice, validate the localization mechanism, `zh-CN`/`en-US` coverage, Chinese
   IME, and bilingual layout before broadening the UI surface.

Recheck current GPUI/native packaging and Rust localization ecosystem constraints at the Desktop
implementation boundary rather than encoding stale library assumptions in this Deferred Story.

## State / Status Owners

- Story status and acceptance: this file.
- Refined product/architecture baseline: `docs/proposals/talos-desktop-goal-oriented-workspace.md`.
- Visual baseline: `docs/design/talos-desktop/DESIGN.md`.
- Internationalization baseline: `docs/design/talos-desktop/I18N.md`.
- Remote request state and discussion: GitHub Issue #29.
- Current operating view: `docs/BOARD.md`.
- Compact selection view: `docs/backlog/PRODUCT-BACKLOG.md`.

## User-Facing Documentation

Update user or SDK documentation only when observable behavior or a public integration contract
changes. Do not present Desktop, Work Graph, independent Goal evaluation, or bilingual Desktop UI as
shipped while this Story remains Deferred and the applicable implementation slices have not landed.

When Desktop ships, user-facing setup/settings documentation must explain language selection,
system-language behavior, fallback, and any restart requirement for changing locale.

## Required Reads

- docs/proposals/talos-desktop-goal-oriented-workspace.md
- docs/proposals/talos-desktop.md
- docs/design/talos-desktop/DESIGN.md
- docs/design/talos-desktop/I18N.md
- docs/backlog/active/RUNTIME-001-embeddable-agent-runtime-api.md
- docs/backlog/active/SESSION-009-multi-client-session-architecture.md
- docs/backlog/active/TODO-001-session-todo-list.md
- docs/backlog/active/TODO-002-todo-mutation-reliability.md
- docs/backlog/active/VALIDATION-001-internal-validation-service.md
- docs/decisions/042-embedded-durable-runtime-session-boundary.md
- docs/decisions/052-sdk-publication-and-composition-boundary.md
- crates/talos-runtime/
- crates/talos-session/
- crates/talos-conversation/

## Acceptance For Directional / Documentation Work

- The Desktop product position clearly differs from TUI feature parity.
- The renderer direction is documented as GPUI rather than the obsolete Tauri recommendation.
- The Work Graph/Todo migration direction avoids two planning sources of truth.
- Independent Goal evaluation is required and executor self-report is explicitly insufficient.
- Mission-level evaluation and evaluated Delivery are part of the target workflow.
- The visual direction is light-first, Nord-derived, low-density, and focused on one dominant
  execution narrative.
- Internationalization is a first-class Desktop requirement with initial `zh-CN` and `en-US`
  coverage, locale-neutral domain identity, bilingual layout/IME requirements, and deterministic
  fallback documented before implementation.
- The future prerequisite implementation chain is explicitly decomposed but no slice is created or
  authorized by this documentation refinement.
- No production code or implementation authorization is implied by this Story update.

## Residual Destination

Implementation must use new governed iterations/claims. The first implementation residual is P0 of
the Work Graph/evaluation prerequisite chain defined by the refined design baseline; the first GPUI
Desktop implementation is a later independent slice after the full chain is merged and must include
the internationalization foundation for its visible UI scope.
