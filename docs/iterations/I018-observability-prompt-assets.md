# I018: Observability and Prompt Assets

**User can**: Run Talos with bounded local log files and review built-in prompt behavior as
standalone embedded assets.

## Status: Planned

## Scope

This iteration handles two small but boundary-sensitive infrastructure changes that should land
before memory and exploration make logs and prompts more important.

## Selected Stories

- [ ] #ARCH-S8 R2: file logging with rotation and retention under ADR-014.
- [ ] #I018-S1: extract built-in prompt text into compile-time embedded prompt assets under ADR-015.

## Acceptance Criteria

- [ ] TUI log output remains file-backed and cannot grow without configured bounds.
- [ ] `[log.file]` supports enabled/path/max-size/max-files/rotation configuration.
- [ ] Non-TUI modes still work with stderr-only logging by default.
- [ ] Built-in prompt assets live in standalone files and are embedded at compile time.
- [ ] Tests verify required prompt assets are present and non-empty.
- [ ] `cargo test -p talos-config -p talos-cli -p talos-agent` passes.

## Out of Scope

- JSON log contracts and shared tracing spans (#ARCH-S8 R3).
- Runtime user-editable prompt packs.
- Memory or exploration prompt behavior beyond creating asset boundaries.

