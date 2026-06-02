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
| I009 extensibility must not bypass permissions | ADR-009 provenance; I009 execution record | #I009-S3/S4/S5; I009 Review | Backend/runtime implemented; TUI consumer work pending | TUI provenance markers and `/plugins` command land or move through change control |
| Codex-like terminal experience | ADR-005 / ADR-006 session seam; reference project Codex patterns | #I010-S7; I010 R2/R3 | Planned | Full-screen and inline/no-alt-screen modes share one session event stream; scrollback-preserving mode verified |
| Native POSIX-style basic tools to reduce host environment dependency | No ADR yet; likely needed if tool provenance or public API expands | #I012-S1; I012 Portable Tools | Planned | Native POSIX subset works on minimal `PATH`; write tools remain permission-gated |
| Embeddable local tool packs linked to pluginized tools | ADR-009 provenance; future plugin registration design | #I012-S2 | Planned | Native POSIX pack registers through same tool-pack path future local plugins can use |
| Provider openness without recompilation | Provider plugin proposal | #I011-S1 implemented; #I011-S2 backlog | S1 implemented; S2 planned | Configurable provider schema and migration path are implemented without hard-coded provider variants |
| Self-evolution runtime wiring | ADR-001; ADR-005 hook-driven evolution clarification | I008 Active | Implemented, awaiting review close | I008 review evidence confirms all runtime paths behave correctly and status moves to Complete |

## Open Documentation Corrections

- I009 remains **Review**, not Complete, until consumer-side TUI work is closed or
  formally moved to a new story.
- I012 is a planned requirement created from the environment-dependency reduction goal;
  it must not be implemented opportunistically inside unrelated polish work.
- Any expansion of native POSIX tools beyond the initial subset should go through
  backlog change control, because it can easily grow into a shell replacement.

## Operating Rule

When a new requirement is accepted, update this document in the same session as the
backlog or iteration change. If implementation lands, update the row from Planned or
Review to Implemented only after verification evidence is recorded.
