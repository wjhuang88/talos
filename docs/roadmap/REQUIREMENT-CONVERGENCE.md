# Requirement Convergence

## Purpose

Every requirement, proposal, and technical decision must converge on an implementation
path or be explicitly marked as deferred. A requirement is not "done" because it is
written down; it is done only when the backlog story, iteration record, implementation,
verification evidence, and public status all agree.

## Closure Path

1. **Intake**: record the requirement in backlog, proposal, or iteration planning.
2. **Decision**: create or reference an ADR when the requirement changes architecture,
   hard constraints, security boundaries, or long-term public API.
3. **Story**: assign a story ID with acceptance criteria and non-goals.
4. **Iteration**: select the story into an iteration document with status.
5. **Implementation**: land code and tests without bypassing hard constraints.
6. **Verification**: record commands, test results, and runtime evidence.
7. **Status Sync**: update README, iteration index, backlog status, and ADR index.

## Current Requirement Map

| Requirement | Decision / Reference | Backlog / Iteration | Current State | Closure Condition |
|---|---|---|---|---|
| Bundled SQLite must be static/self-contained and not require system SQLite | ADR-008; ADR-002 storage architecture | I006 SQLite search; README storage note | Implemented and documented | Keep `rusqlite/bundled`; verify binary has no system SQLite dependency when release packaging is cut |
| I008/I009 review closure before new mainline work | I008/I009 execution records; R1 review closure | R1 Review Closure | Complete (2026-06-03) | I008/I009 moved to Complete; I009 TUI consumer work deferred to #I009-S6; I010 R2 completed and R3 is next |
| I009 extensibility must not bypass permissions | ADR-009 provenance; I009 execution record | #I009-S3/S4/S5; #I009-S6 (deferred TUI consumer) | Backend/runtime complete; TUI consumer deferred to #I009-S6 | #I009-S6 lands in I010 R2/R3 or a dedicated follow-up |
| Codex-like terminal experience | ADR-005 / ADR-006 session seam; reference project Codex patterns | #I010-S7; I010 R2/R3 | R2 implemented (2026-06-03); R3 polish pending | Full-screen and inline/no-alt-screen modes share one session event stream; scrollback-preserving mode verified; R3 completes markdown, diff, slash command, and visual polish |
| Native POSIX-style basic tools to reduce host environment dependency | ADR required before implementation if tool-pack, provenance, config, or public tool listing changes | #I012-S1; I012 Portable Tools | Planned | Native POSIX subset works on minimal `PATH`; write tools remain permission-gated |
| Embeddable local tool packs linked to pluginized tools | ADR-009 provenance; future plugin registration design; likely I012 ADR | #I012-S2 | Planned | ADR records native tool-pack boundary; native POSIX pack registers through same path future local plugins can use |
| Built-in workspace search without host utilities | ADR-010; built-in search proposal; fff reference design | #I012-S3 | Planned | `find_files` and `grep` work without host `find`/`grep`/`rg`, stay workspace-bounded, respect ignore rules, and enforce output budgets |
| Self-contained Git operations without `git2`/libgit2 | ADR-010 | #I012-S4 | Planned | Read-only Git tools target `gix`; host `git` is fallback/temporary bridge only; `git2`/libgit2 deferred unless a later ADR approves native dependency |
| Provider openness without recompilation | Provider plugin proposal; ADR required before #I011-S2 implementation | #I011-S1 implemented; #I011-S2 backlog | S1 implemented; S2 paused/deferred | Configurable provider schema and migration path are implemented without hard-coded provider variants; S2 resumes after R1/I010 or explicit priority change |
| AI-mediated approval policy | ADR required before implementation | #I010-S6 Guardian AI sub-agent | Planned/deferred | Guardian approval scope, denial/default behavior, audit trail, prompt privacy, and permission-pipeline composition are recorded before any auto-approval code lands |
| Exec approval policy DSL | ADR required before implementation | #I010-S8 Exec policy DSL rules | Planned/deferred | DSL grammar, matching order, bypass behavior, path/env handling, and `PermissionDecision` mapping are recorded before rule files affect approvals |
| Production logging outputs | ADR required before R2/R3; R1 may proceed if centralized-only | #ARCH-S8 Production-grade logging system | Planned | R1 centralizes existing tracing init; file output/rotation/JSON/span contracts wait for ADR |
| Reasoning/thinking provider fields | Reasoning proposal; ADR required before story implementation | Future story after #I011-S2 or explicit activation | Proposal | Provider request schema, stream event shape, persistence, TUI/RPC exposure, and evolution hook impact are recorded before code |
| Self-evolution runtime wiring | ADR-001; ADR-005 hook-driven evolution clarification | I008 Complete | Complete (2026-06-03) | Hook-based EvolutionHookHandler registered uniformly across all paths; runtime evidence recorded |

## Closed Documentation Corrections

- I008 is **Complete** (2026-06-03). Hook-based runtime evidence recorded; all paths verified.
- I009 is **Complete** (2026-06-03). Backend/runtime extensibility shipped. TUI consumer work
  (provenance markers + `/plugins`) deferred to `#I009-S6` through change control.
- R1 Review Closure is **Complete** (2026-06-03). I010 R2 Architecture Convergence is complete; I010 R3 Product Polish is next.
- I011 S2 remains paused; do not treat provider plugin architecture as active work
  without a priority-change update.
- I012 is a planned requirement created from the environment-dependency reduction goal;
  it now includes POSIX-style tools, workspace search, and structured Git tools. It must not be
  implemented opportunistically inside unrelated polish work.
- Any expansion of native POSIX tools beyond the initial subset should go through
  backlog change control, because it can easily grow into a shell replacement.
- Any native Git/search dependency beyond the ADR-010 `gix` direction (`git2`, persistent search
  DB, watcher, frecency store, content index) must follow ADR-010 and get an explicit follow-up
  decision.
- Guardian auto-approval and exec policy DSL must not start implementation without dedicated ADRs;
  both can change the effective permission boundary.
- Provider plugin S2 and reasoning/thinking support must not use proposals alone as execution
  authority when they change config schema, provider protocol, persisted session fields, or public
  stream/RPC events.
- Logging R1 can centralize current subscriber setup, but file output, rotation, JSON logs, span
  contracts, or a shared logging crate require an ADR first.

## Operating Rule

When a new requirement is accepted, update this document in the same session as the
backlog or iteration change. If implementation lands, update the row from Planned or
Review to Implemented only after verification evidence is recorded.

## Pre-Commit Sync Rule

Before committing any change that modifies implementation, README, backlog, iterations, ADRs, or
roadmap content, check whether an affected requirement row already exists here. If it does, sync the
state and closure condition. If it does not, add a row or record why the change does not create a
tracked requirement.

Rows may move from Planned or Review only after verification evidence is recorded in the relevant
iteration document.
