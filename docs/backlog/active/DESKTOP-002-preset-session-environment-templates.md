# DESKTOP-002: Preset Session Environment Templates

| Field | Value |
|---|---|
| Story ID | DESKTOP-002 |
| Type | Product / Domain Epic |
| Priority | P1 |
| Status | Blocked / Unclaimed |
| Source | [GitHub Issue #308](https://github.com/wjhuang88/talos/issues/308) |
| Selected Iteration | None |
| Depends On | MODEL-012/#146 multi-model role contract; SESSION-009 shared Session authority; DESKTOP-001 prerequisites |

## Collaboration Claim

| Field | Value |
|---|---|
| Claim State | Unclaimed |
| Responsible Actor | Not assigned |
| Executing Agent | Not assigned |
| Work Slice | Not assigned |
| Claimed At | Not applicable |
| Source Issue | #308 |
| Governance Claim PR | Not applicable |
| Authorization Mode | Not applicable |
| Authorization Evidence | Not applicable |
| Implementation PR | Not started |
| Last Updated | 2026-08-19 |
| Handoff / Release Condition | Resolve the canonical model-role and shared Session-environment contracts, then decompose this Epic into separately runnable domain and Desktop product Stories. No Desktop or shared-runtime implementation is authorized. |

## Identity / Goal / Value

Make a Preset the user-facing template for creating a complete Talos Session work environment,
combining a canonical model profile with extension bindings without creating a live parent-child
binding between the Preset and an existing Session.

## Product And Domain Boundary

- Every new Session is instantiated from exactly one Preset, using a real built-in default when the
  user does not choose another.
- Preset application is a one-time snapshot/copy into Session-owned environment state.
- Existing Sessions never resolve their runtime configuration through a live `preset_id`; optional
  `created_from` data is provenance only.
- A Session may change models/extensions independently, save its environment as a new Preset or
  explicitly replace it by applying another Preset.
- User-facing Presets contain a canonical model-role profile and unified Extensions projection;
  Skill/MCP/Hook/Plugin remain implementation types and advanced metadata.

## Blocking Decisions And Dependencies

- MODEL-012/#146 must decide canonical model roles, routing, fallback, capability admission,
  Session-scoped ownership and compatibility with today's single-model configuration. The
  `Default / Quick / Deep` labels remain a product hypothesis until that owner accepts them.
- SESSION-009 must preserve one shared Session authority; DESKTOP-002 cannot create a Desktop-only
  environment owner or model router.
- DESKTOP-001's governed prerequisite chain must supply shared Work/Evaluation/runtime contracts
  before product implementation; this Epic cannot bypass that chain.
- Snapshot identity, extension configuration, provenance, migration and rollback require explicit
  domain decisions before any schema or public API change.

## Required Child Decomposition

1. Preset and Session Environment snapshot/migration contract.
2. Canonical model-profile consumption after MODEL-012 resolves the role contract.
3. Extension projection and safe binding identity contract.
4. Preset Library/editor and extension picker.
5. New-Session/default-Preset integration.
6. Apply-Preset and save-Session-as-Preset flows.
7. Desktop visual, i18n, keyboard, accessibility and end-to-end validation.

Each implementation child requires its own owner, runnable iteration, effective Collaboration
Claim, exact-head gates and evidence. Shared domain/runtime work belongs to mainline; Desktop owns
only the product host and presentation after its prerequisites are complete.

## Exclusions

- No implementation, GPUI/native renderer, new crate/dependency or Cargo change.
- No persistence/schema migration or public runtime API change.
- No Preset revision binding, inheritance tree, multi-Preset layering or automatic merge.
- No permission grant, sandbox weakening or authority expansion through extension membership.
- No marketplace, sharing, cloud sync, organization policy or analytics/dashboard work.

## Acceptance For Epic Refinement

- [ ] MODEL-012/#146 establishes the canonical multi-model role and routing contract.
- [ ] Session Environment authority and snapshot lifecycle are explicit and compatible with
      SESSION-009.
- [ ] Extension identity/configuration/provenance and permission invariants are testable.
- [ ] Migration, rollback and built-in-default behavior are specified without live Preset binding.
- [ ] Domain/runtime and Desktop responsibilities are separated into runnable child Stories.
- [ ] The first child is Ready with an effective Collaboration Claim before implementation.

## State / Status Owners

- Epic scope, dependencies and decomposition: this file.
- Product discussion and acceptance detail: GitHub Issue #308.
- Compact planning view: `docs/backlog/PRODUCT-BACKLOG.md`.
- Derived operating view: `docs/BOARD.md`.

## User-Facing Documentation

Future product implementation must update Desktop design and user documentation in both `zh-CN`
and `en-US`. This intake changes no user-visible behavior.

## Required Reads

- `docs/backlog/active/DESKTOP-001-desktop-product-direction.md`
- `docs/backlog/active/MODEL-012-utility-model-role-and-bounded-routing.md`
- `docs/backlog/active/SESSION-009-multi-client-session-architecture.md`
- `docs/proposals/talos-desktop-goal-oriented-workspace.md`
- `docs/design/talos-desktop/DESIGN.md`
