# I018: Observability and Prompt Assets

**User can**: Run Talos with bounded local log files and review built-in prompt behavior as
standalone embedded assets.

## Status: Complete — fulfilled by I047 (reconciled 2026-07-20)

I047 delivered the same bounded log + embedded prompt asset acceptance as prerequisite closure for
I019. This preserved baseline is fulfilled by I047; no separate I018 implementation remains.

## Scope

This iteration handles two small but boundary-sensitive infrastructure changes that should land
before memory and exploration make logs and prompts more important.

## Selected Stories

- [x] #ARCH-S8 R2: file logging with rotation and retention under ADR-014.
- [x] #I018-S1: extract built-in prompt text into compile-time embedded prompt assets under ADR-015.

## Acceptance Criteria

- [x] TUI log output remains file-backed and cannot grow without configured bounds.
- [x] `[log.file]` supports enabled/path/max-size/max-files/rotation configuration.
- [x] Non-TUI modes still work with stderr-only logging by default.
- [x] Built-in prompt assets live in standalone files and are embedded at compile time.
- [x] Tests verify required prompt assets are present and non-empty.
- [x] I047 locked validation passed.

## Out of Scope

- JSON log contracts and shared tracing spans (#ARCH-S8 R3).
- Runtime user-editable prompt packs.
- Memory or exploration prompt behavior beyond creating asset boundaries.
