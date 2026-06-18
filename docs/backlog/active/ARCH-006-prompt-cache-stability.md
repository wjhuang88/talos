# ARCH-006: Prompt Cache Stability

**Status**: Complete (→ I029, 2026-06-18)
**Priority**: P2
**Source**: ARCH-002 audit and I026 prompt/cache work
**Depends on**: I026 complete

## Problem

I026 implemented dynamic prompt templates and Anthropic cache-control emission. Remaining work is
to make the cache-stable prefix an explicit session contract and expose enough metadata to debug
cache misses.

## Scope

- Represent the system prompt prefix as a session-stable snapshot.
- Verify tool, skill, and context sections do not mutate mid-session under normal CLI/TUI startup.
- Surface provider cache metadata where available.
- Keep OpenAI-compatible request ordering stable.

## Acceptance Criteria

- [x] System prompt prefix is computed once per session and reused across turns.
- [x] Tests prove tool/skill/context sections do not change mid-session unless the session is
      explicitly rebuilt.
- [x] Anthropic cache hit/miss metadata is captured or explicitly unavailable in provider output.
- [x] OpenAI-compatible providers keep system messages first.
- [x] `cargo test -p talos-agent -p talos-provider` passes.

## Verification Notes

Do not add provider-specific cache behavior to generic core traits unless another provider needs
the same concept.

- 2026-06-18: Completed in I029. Stable prefix caching landed in `talos-agent`; Anthropic
  cache-control remains provider-local; OpenAI-compatible message ordering remains stable.
  `cargo test -p talos-agent` passed with 146 tests, and workspace clippy remained clean.
