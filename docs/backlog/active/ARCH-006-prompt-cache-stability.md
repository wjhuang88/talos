# ARCH-006: Prompt Cache Stability

**Status**: Planned
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

- [ ] System prompt prefix is computed once per session and reused across turns.
- [ ] Tests prove tool/skill/context sections do not change mid-session unless the session is
      explicitly rebuilt.
- [ ] Anthropic cache hit/miss metadata is captured or explicitly unavailable in provider output.
- [ ] OpenAI-compatible providers keep system messages first.
- [ ] `cargo test -p talos-agent -p talos-provider` passes.

## Verification Notes

Do not add provider-specific cache behavior to generic core traits unless another provider needs
the same concept.
