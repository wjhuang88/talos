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
| I008/I009 review closure before new mainline work | I008/I009 execution records; R1 review closure | R1 Review Closure; I008 Review; I009 Review | Active | I008/I009 move to Complete or residual work is moved through change control; I011 S2 stays paused; I010 R2 is then activated |
| I009 extensibility must not bypass permissions | ADR-009 provenance; I009 execution record | #I009-S3/S4/S5; I009 Review | Backend/runtime implemented; TUI consumer work pending under R1 | TUI provenance markers and `/plugins` command land or move through change control |
| Codex-like terminal experience | ADR-005 / ADR-006 session seam; reference project Codex patterns | #I010-S7; I010 R2/R3 | Planned after R1 closure | R2 first: full-screen and inline/no-alt-screen modes share one session event stream; scrollback-preserving mode verified before R3 polish |
| Native POSIX-style basic tools to reduce host environment dependency | ADR required before implementation if tool-pack, provenance, config, or public tool listing changes | #I012-S1; I012 Portable Tools | Planned | Native POSIX subset works on minimal `PATH`; write tools remain permission-gated |
| Embeddable local tool packs linked to pluginized tools | ADR-009 provenance; future plugin registration design; likely I012 ADR | #I012-S2 | Planned | ADR records native tool-pack boundary; native POSIX pack registers through same path future local plugins can use |
| Provider openness without recompilation | Provider plugin proposal | #I011-S1 implemented; #I011-S2 backlog | S1 implemented; S2 paused/deferred | Configurable provider schema and migration path are implemented without hard-coded provider variants; S2 resumes after R1/I010 or explicit priority change |
| Self-evolution runtime wiring | ADR-001; ADR-005 hook-driven evolution clarification | I008 Review | Implemented, awaiting review close | I008 review evidence confirms all runtime paths behave correctly and status moves to Complete |

## Open Documentation Corrections

- I009 remains **Review**, not Complete, until consumer-side TUI work is closed or
  formally moved to a new story.
- R1 is the active operating round. It exists to close I008/I009 Review drift before I010 R2 starts.
- I011 S2 is paused while R1 is active; do not treat provider plugin architecture as active work
  without a priority-change update.
- I012 is a planned requirement created from the environment-dependency reduction goal;
  it must not be implemented opportunistically inside unrelated polish work.
- Any expansion of native POSIX tools beyond the initial subset should go through
  backlog change control, because it can easily grow into a shell replacement.

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
